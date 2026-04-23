//! Filter model used by [`crate::Swdir::walk`].
//!
//! # Overview
//!
//! Filtering in 0.10 is built around **one** type, [`FilterRule`], and its
//! output, [`Decision`]. Each rule inspects a directory entry through a
//! [`FilterContext`] and returns a [`Decision`] with two independent axes:
//!
//! * `include` — "should this entry appear in the result tree?"
//! * `descend` — "should the walker keep going into this entry's children?"
//!
//! Multiple rules combine with **AND**: a rule can only be more restrictive,
//! never more permissive. An entry is included iff every rule's decision
//! says `include = true`; a directory is descended iff every rule says
//! `descend = true`.
//!
//! The two axes are independent on purpose. A common case — "I don't want
//! hidden dirs in my output, but I still want to see inside them" — is
//! expressible because each [`FilterRule`] decides the two axes separately.
//! The default [`FilterRule::SkipHidden`] returns `include = false, descend
//! = false` for hidden entries (the conventional meaning); callers who want
//! the more permissive variant can write their own rule.
//!
//! # Feature gate
//!
//! Everything in this module is behind the default-on `filter` feature. If
//! you disable default features, [`crate::Swdir`] still works — it just
//! returns every entry it can read.
//!
//! # Not in 0.10
//!
//! * No `advanced-filter` feature. The enum is [`#[non_exhaustive]`] so new
//!   variants can be added later without breaking callers.
//! * No regex / glob / metadata / user-predicate rules. The basics cover
//!   the overwhelming majority of real-world needs.

use std::path::{Path, PathBuf};

use crate::helpers::error::SwdirError;

/// Kind of filesystem entry. Mirrors [`std::fs::FileType`] but as a plain,
/// pattern-matchable enum.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryKind {
    /// A regular file.
    File,
    /// A directory.
    Dir,
    /// A symbolic link (not followed).
    Symlink,
}

impl EntryKind {
    /// Classify a [`std::fs::FileType`] into an [`EntryKind`].
    ///
    /// Symlinks take precedence over the dir/file classification, matching
    /// the behavior of [`std::fs::DirEntry::file_type`] (which does not
    /// follow links).
    pub fn from_file_type(ft: std::fs::FileType) -> Option<Self> {
        if ft.is_symlink() {
            Some(Self::Symlink)
        } else if ft.is_dir() {
            Some(Self::Dir)
        } else if ft.is_file() {
            Some(Self::File)
        } else {
            None
        }
    }
}

/// What to do with a given entry.
///
/// `include` and `descend` are independent. Common combinations:
///
/// * `Decision::PASS` — include in results AND descend into children.
/// * `Decision::DROP` — exclude from results AND don't descend.
/// * `{ include: false, descend: true }` — hide directory from output but
///   still look inside (e.g. the `OnlyKinds(File)` rule does this for
///   directories so that files in them remain reachable).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Decision {
    /// Whether this entry should appear in the result tree.
    pub include: bool,
    /// Whether the walker should keep descending into this entry's children.
    /// Meaningful only for directories; files/symlinks ignore it.
    pub descend: bool,
}

impl Decision {
    /// Include the entry and keep descending.
    pub const PASS: Self = Self {
        include: true,
        descend: true,
    };

    /// Exclude the entry and stop descending.
    pub const DROP: Self = Self {
        include: false,
        descend: false,
    };

    /// Hide the entry from results but keep descending through it. Useful
    /// for "only files, please" where the containing directories should
    /// still be traversed so the files can be found.
    pub const HIDDEN_PASSTHROUGH: Self = Self {
        include: false,
        descend: true,
    };

    /// Construct a `Decision` from the two axes directly.
    pub const fn new(include: bool, descend: bool) -> Self {
        Self { include, descend }
    }

    /// Combine two decisions with logical AND on each axis.
    ///
    /// This is how the walker merges decisions from multiple rules: an
    /// entry is only included / descended if *every* rule agrees.
    pub const fn and(self, other: Self) -> Self {
        Self {
            include: self.include && other.include,
            descend: self.descend && other.descend,
        }
    }
}

/// Context supplied to a filter rule when evaluating a candidate entry.
///
/// Construction is crate-internal; callers receive this inside their
/// [`FilterRule::evaluate`] logic. The fields are `pub` for reading but
/// the struct itself is not meant to be built from outside.
#[derive(Debug)]
pub struct FilterContext<'a> {
    /// Full path of the entry, rooted at whatever the caller passed to
    /// [`crate::Swdir::root_path`].
    pub path: &'a Path,
    /// Classified type of the entry.
    pub kind: EntryKind,
    /// Depth from the scan root. The root's direct children are at depth 0.
    pub depth: usize,
    /// Whether the entry matches the platform's "hidden" convention.
    pub is_hidden: bool,
}

impl<'a> FilterContext<'a> {
    /// Crate-internal constructor.
    pub(crate) fn new(path: &'a Path, kind: EntryKind, depth: usize, is_hidden: bool) -> Self {
        Self {
            path,
            kind,
            depth,
            is_hidden,
        }
    }
}

/// A single filter condition.
///
/// Rules are composed by storing several in [`crate::Swdir`]; at walk
/// time the walker calls [`FilterRule::evaluate`] on each and ANDs the
/// results via [`Decision::and`].
///
/// The enum is `#[non_exhaustive]` so that future extensions (e.g. a
/// name-matching rule) don't break existing `match` statements. Prefer
/// the associated constructors below over building variants directly —
/// they perform validation where appropriate.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum FilterRule {
    /// Skip hidden entries entirely (neither include nor descend).
    /// `Swdir::default()` installs exactly one of these.
    SkipHidden,

    /// Include only entries of one of the listed kinds. Directories that
    /// don't match are still *descended* so that their contents can
    /// satisfy the rule — a common expectation when saying "give me only
    /// the files under here".
    OnlyKinds(Vec<EntryKind>),

    /// Include only files whose extension (case-sensitive) is in the list.
    /// Non-file entries pass this rule unchanged.
    ExtensionAllowlist(Vec<String>),

    /// Exclude files whose extension (case-sensitive) is in the list.
    /// Non-file entries pass this rule unchanged.
    ExtensionDenylist(Vec<String>),

    /// Restrict results to entries under this path prefix. Directories
    /// that could *contain* matching entries (i.e. the prefix starts with
    /// them) are still descended.
    UnderPath(PathBuf),

    /// Skip everything under this path prefix — entries matching the
    /// prefix are neither included nor descended into.
    NotUnderPath(PathBuf),

    /// Cap the depth of included/descended entries. `MaxDepth(n)`:
    /// include entries at `depth <= n`, descend at `depth < n`.
    /// Overlaps [`crate::Recurse::Depth`] — using either is fine; both
    /// together AND together, so the stricter limit wins.
    MaxDepth(usize),
}

impl FilterRule {
    // --- validated constructors ---------------------------------------------

    /// Build a [`FilterRule::SkipHidden`]. Zero-argument convenience.
    pub fn skip_hidden() -> Self {
        Self::SkipHidden
    }

    /// Build a [`FilterRule::OnlyKinds`] matching a single kind.
    pub fn only_kind(kind: EntryKind) -> Self {
        Self::OnlyKinds(vec![kind])
    }

    /// Build a [`FilterRule::OnlyKinds`] from any iterator of kinds.
    pub fn only_kinds<I: IntoIterator<Item = EntryKind>>(kinds: I) -> Self {
        Self::OnlyKinds(kinds.into_iter().collect())
    }

    /// Build a validated [`FilterRule::ExtensionAllowlist`].
    ///
    /// Returns [`SwdirError::InvalidExtensionListItem`] if any entry begins
    /// with `'.'` — [`Path::extension`] never includes the leading dot,
    /// so such entries would silently never match.
    pub fn extension_allowlist<S, I>(exts: I) -> Result<Self, SwdirError>
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        let list = normalize_extensions(exts)?;
        Ok(Self::ExtensionAllowlist(list))
    }

    /// Build a validated [`FilterRule::ExtensionDenylist`]. See
    /// [`FilterRule::extension_allowlist`] for the validation contract.
    pub fn extension_denylist<S, I>(exts: I) -> Result<Self, SwdirError>
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        let list = normalize_extensions(exts)?;
        Ok(Self::ExtensionDenylist(list))
    }

    /// Build a [`FilterRule::UnderPath`].
    pub fn under_path<P: Into<PathBuf>>(path: P) -> Self {
        Self::UnderPath(path.into())
    }

    /// Build a [`FilterRule::NotUnderPath`].
    pub fn not_under_path<P: Into<PathBuf>>(path: P) -> Self {
        Self::NotUnderPath(path.into())
    }

    /// Build a [`FilterRule::MaxDepth`].
    pub fn max_depth(n: usize) -> Self {
        Self::MaxDepth(n)
    }

    // --- evaluation ----------------------------------------------------------

    /// Apply this rule to an entry and compute its [`Decision`].
    ///
    /// See the module docs for how multiple rules combine. Callers who
    /// don't need to inspect rules manually can ignore this method and
    /// just hand their rules to [`crate::Swdir::filter`].
    pub fn evaluate(&self, ctx: &FilterContext<'_>) -> Decision {
        match self {
            Self::SkipHidden => {
                if ctx.is_hidden {
                    Decision::DROP
                } else {
                    Decision::PASS
                }
            }

            Self::OnlyKinds(kinds) => {
                if kinds.contains(&ctx.kind) {
                    Decision::PASS
                } else {
                    // Still descend into unmatched directories so that
                    // their children can satisfy the rule.
                    if ctx.kind == EntryKind::Dir {
                        Decision::HIDDEN_PASSTHROUGH
                    } else {
                        Decision::DROP
                    }
                }
            }

            Self::ExtensionAllowlist(exts) => {
                if ctx.kind != EntryKind::File {
                    return Decision::PASS;
                }
                if extension_matches(ctx.path, exts) {
                    Decision::PASS
                } else {
                    Decision::DROP
                }
            }

            Self::ExtensionDenylist(exts) => {
                if ctx.kind != EntryKind::File {
                    return Decision::PASS;
                }
                if extension_matches(ctx.path, exts) {
                    Decision::DROP
                } else {
                    Decision::PASS
                }
            }

            Self::UnderPath(prefix) => {
                let under = ctx.path.starts_with(prefix);
                if under {
                    Decision::PASS
                } else {
                    // If the prefix is a descendant of this entry, keep
                    // descending so we eventually reach the target.
                    let maybe_ancestor = ctx.kind == EntryKind::Dir && prefix.starts_with(ctx.path);
                    Decision::new(false, maybe_ancestor)
                }
            }

            Self::NotUnderPath(prefix) => {
                if ctx.path.starts_with(prefix) {
                    Decision::DROP
                } else {
                    Decision::PASS
                }
            }

            Self::MaxDepth(max) => Decision {
                include: ctx.depth <= *max,
                descend: ctx.depth < *max,
            },
        }
    }
}

// ----- helpers --------------------------------------------------------------

fn normalize_extensions<S, I>(exts: I) -> Result<Vec<String>, SwdirError>
where
    S: Into<String>,
    I: IntoIterator<Item = S>,
{
    let list: Vec<String> = exts.into_iter().map(Into::into).collect();
    for e in &list {
        if e.starts_with('.') {
            return Err(SwdirError::InvalidExtensionListItem(e.clone()));
        }
    }
    Ok(list)
}

fn extension_matches(path: &Path, exts: &[String]) -> bool {
    match path.extension() {
        Some(ext) => {
            let ext = ext.to_string_lossy();
            exts.iter().any(|e| e.as_str() == ext.as_ref())
        }
        None => false,
    }
}

/// Reduce a slice of rules to one combined [`Decision`] by ANDing them
/// pairwise. With no rules this is [`Decision::PASS`] — matching the
/// "no filter means keep everything" intuition.
pub(crate) fn evaluate_all(rules: &[FilterRule], ctx: &FilterContext<'_>) -> Decision {
    rules
        .iter()
        .fold(Decision::PASS, |acc, r| acc.and(r.evaluate(ctx)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx<'a>(path: &'a Path, kind: EntryKind, depth: usize, hidden: bool) -> FilterContext<'a> {
        FilterContext::new(path, kind, depth, hidden)
    }

    #[test]
    fn decision_and_passes_only_when_both_pass() {
        assert_eq!(Decision::PASS.and(Decision::PASS), Decision::PASS);
        assert_eq!(Decision::PASS.and(Decision::DROP), Decision::DROP);
        let hp = Decision::HIDDEN_PASSTHROUGH;
        assert_eq!(Decision::PASS.and(hp), hp);
        assert_eq!(hp.and(hp), hp);
    }

    #[test]
    fn skip_hidden_drops_hidden_entries() {
        let r = FilterRule::skip_hidden();
        let p = PathBuf::from(".hidden");
        let c = ctx(&p, EntryKind::File, 0, true);
        assert_eq!(r.evaluate(&c), Decision::DROP);

        let p2 = PathBuf::from("visible");
        let c2 = ctx(&p2, EntryKind::File, 0, false);
        assert_eq!(r.evaluate(&c2), Decision::PASS);
    }

    #[test]
    fn only_kinds_keeps_dirs_traversable() {
        let r = FilterRule::only_kind(EntryKind::File);
        let p = PathBuf::from("some/dir");
        let c = ctx(&p, EntryKind::Dir, 0, false);
        // dir itself is dropped, but descent continues
        assert_eq!(r.evaluate(&c), Decision::HIDDEN_PASSTHROUGH);
    }

    #[test]
    fn extension_allowlist_validates_leading_dot() {
        let err = FilterRule::extension_allowlist([".md"]).unwrap_err();
        assert_eq!(err, SwdirError::InvalidExtensionListItem(".md".into()));
    }

    #[test]
    fn extension_allowlist_drops_non_matching() {
        let r = FilterRule::extension_allowlist(["md"]).unwrap();
        let p = PathBuf::from("a.txt");
        let c = ctx(&p, EntryKind::File, 0, false);
        assert_eq!(r.evaluate(&c), Decision::DROP);
    }

    #[test]
    fn max_depth_splits_include_and_descend() {
        let r = FilterRule::max_depth(1);
        let p = PathBuf::from("x");
        let at_1 = r.evaluate(&ctx(&p, EntryKind::File, 1, false));
        assert_eq!(at_1, Decision::new(true, false));
        let at_2 = r.evaluate(&ctx(&p, EntryKind::File, 2, false));
        assert_eq!(at_2, Decision::DROP);
    }

    #[test]
    fn under_path_allows_ancestor_descent() {
        let r = FilterRule::under_path("/a/b");
        let anc = PathBuf::from("/a");
        let c = ctx(&anc, EntryKind::Dir, 0, false);
        let d = r.evaluate(&c);
        assert!(!d.include);
        assert!(d.descend);
    }
}
