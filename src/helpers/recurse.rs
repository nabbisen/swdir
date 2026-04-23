//! Recursion policy for [`crate::Swdir`].
//!
//! In 0.9 this was a two-field struct (`enabled` + `depth_limit`) which
//! allowed the nonsensical combination `enabled = false, depth_limit = Some(..)`
//! and made reading call sites hard. In 0.10 it is an enum that can only
//! express the three meaningful states:
//!
//! * [`Recurse::None`] — don't descend at all (root dir only)
//! * [`Recurse::Unlimited`] — descend the whole tree
//! * [`Recurse::Depth(n)`] — descend at most `n` levels below the root
//!
//! For further refinement by *filter* (rather than by the traversal policy
//! itself), see [`crate::FilterRule::MaxDepth`].

/// Recursion policy controlling how deeply [`crate::Swdir::walk`] descends.
///
/// Depth is counted from the scan root. `Depth(0)` behaves the same as
/// [`Recurse::None`]: root-level entries are listed but no subdirectory is
/// entered. `Depth(1)` enters the root's immediate child directories but no
/// further, and so on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Recurse {
    /// Don't descend into any subdirectory.
    ///
    /// Matches the 0.9 default ("don't recurse") so common callers don't
    /// get surprising behavior just by bumping major.
    #[default]
    None,
    /// Descend without a depth cap.
    Unlimited,
    /// Descend at most `n` levels below the scan root.
    Depth(usize),
}

impl Recurse {
    /// Convenience constructor equivalent to [`Recurse::None`].
    pub fn none() -> Self {
        Self::None
    }

    /// Convenience constructor equivalent to [`Recurse::Unlimited`].
    pub fn unlimited() -> Self {
        Self::Unlimited
    }

    /// Convenience constructor equivalent to [`Recurse::Depth`].
    pub fn depth(n: usize) -> Self {
        Self::Depth(n)
    }

    /// `true` if the policy allows any descent.
    pub fn is_enabled(self) -> bool {
        !matches!(self, Self::None)
    }

    /// Inspect the depth cap as `Option<usize>`:
    ///
    /// * `None` means "no descent" or "unlimited" — check [`Recurse::is_enabled`] to tell which.
    /// * `Some(n)` means "descend at most `n` levels".
    pub fn depth_limit(self) -> Option<usize> {
        match self {
            Self::Depth(n) => Some(n),
            _ => None,
        }
    }
}
