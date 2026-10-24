//! Workload hints for bit and packed substring searches.
//!
//! Hints describe expected input properties. They never change search results,
//! and an implementation may ignore a hint when it cannot use it efficiently.

/// Describes which end of a needle is expected to be more selective.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AnchorHint {
    /// No information about needle selectivity is available.
    #[default]
    Unknown,
    /// The needle prefix is expected to reject more candidates.
    Prefix,
    /// The needle suffix is expected to reject more candidates.
    Suffix,
}

/// Describes where matches are expected to occur in the haystack.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LocationHint {
    /// No information about match location is available.
    #[default]
    Unknown,
    /// Matches are expected to occur near the beginning.
    NearStart,
    /// Matches are expected to occur near the end.
    NearEnd,
}

/// Workload hints for searches that return the first or last matching index.
///
/// The anchor is advisory; it does not change the required first or last match.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FindOptions {
    anchor: AnchorHint,
}

impl FindOptions {
    /// Returns options with the expected needle anchor selectivity.
    #[must_use]
    pub const fn with_anchor(mut self, anchor: AnchorHint) -> Self {
        self.anchor = anchor;
        self
    }

    /// Returns the expected needle anchor selectivity.
    #[must_use]
    pub const fn anchor(self) -> AnchorHint {
        self.anchor
    }
}

/// Workload hints for searches that only test whether any match exists.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ContainsOptions {
    anchor: AnchorHint,
    location: LocationHint,
}

impl ContainsOptions {
    /// Returns options with the expected needle anchor selectivity.
    #[must_use]
    pub const fn with_anchor(mut self, anchor: AnchorHint) -> Self {
        self.anchor = anchor;
        self
    }

    /// Returns options with the expected match location.
    #[must_use]
    pub const fn with_location(mut self, location: LocationHint) -> Self {
        self.location = location;
        self
    }

    /// Returns the expected needle anchor selectivity.
    #[must_use]
    pub const fn anchor(self) -> AnchorHint {
        self.anchor
    }

    /// Returns the expected match location.
    #[must_use]
    pub const fn location(self) -> LocationHint {
        self.location
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_default_to_unknown_hints() {
        assert_eq!(FindOptions::default().anchor(), AnchorHint::Unknown);

        let options = ContainsOptions::default();
        assert_eq!(options.anchor(), AnchorHint::Unknown);
        assert_eq!(options.location(), LocationHint::Unknown);
    }

    #[test]
    fn builders_set_hints_without_mutating_the_original() {
        let defaults = ContainsOptions::default();
        let configured = defaults
            .with_anchor(AnchorHint::Suffix)
            .with_location(LocationHint::NearEnd);

        assert_eq!(defaults.anchor(), AnchorHint::Unknown);
        assert_eq!(defaults.location(), LocationHint::Unknown);
        assert_eq!(configured.anchor(), AnchorHint::Suffix);
        assert_eq!(configured.location(), LocationHint::NearEnd);
    }
}
