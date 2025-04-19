//! NAV filter
use crate::error::Error;
use gnss_rs::prelude::Constellation;

/// [NavFilterType] describes complex Navigation condition
/// we may apply to filter.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum NavFilterType {
    /// Healthy SV (suitable for navigation)
    Healthy,
    /// Unhealthy SV (not suitable for navigation)
    Unhealthy,
    /// (In-) testing SV (usually not suitable for navigation)
    Testing,
}

/// Complex [NavFilter]
#[derive(Debug, Clone, PartialEq)]
pub struct NavFilter {
    /// [NavFilterType] we support.
    pub filter: NavFilterType,
    /// Possible targetted constellations
    pub constellations: Vec<Constellation>,
}

impl std::str::FromStr for NavFilter {
    type Err = Error;

    fn from_str(s: &str) -> Result<NavFilter, Error> {
        let mut constellations = Vec::new();

        for item in s.split(':') {
            let trimmed = item.trim();

            for csv in trimmed.split(',') {
                if let Ok(parsed) = Constellation::from_str(csv.trim()) {
                    constellations.push(parsed);
                }
            }

            match trimmed {
                "healthy" => {
                    return Ok(NavFilter {
                        constellations,
                        filter: NavFilterType::Healthy,
                    });
                }
                "unhealthy" => {
                    return Ok(NavFilter {
                        constellations,
                        filter: NavFilterType::Unhealthy,
                    });
                }
                "testing" => {
                    return Ok(NavFilter {
                        constellations,
                        filter: NavFilterType::Testing,
                    });
                }
                _ => {}
            }
        }
        Err(Error::InvalidNavFilter)
    }
}

#[cfg(test)]
mod test {
    use super::{NavFilter, NavFilterType};
    use gnss_rs::prelude::Constellation;
    use std::str::FromStr;

    #[test]
    fn nav_filter_parsing() {
        for (value, expected) in [
            (
                "healthy",
                NavFilter {
                    filter: NavFilterType::Healthy,
                    constellations: vec![],
                },
            ),
            (
                "unhealthy",
                NavFilter {
                    filter: NavFilterType::Unhealthy,
                    constellations: vec![],
                },
            ),
            (
                "testing",
                NavFilter {
                    filter: NavFilterType::Testing,
                    constellations: vec![],
                },
            ),
            (
                "gps:testing",
                NavFilter {
                    filter: NavFilterType::Testing,
                    constellations: vec![Constellation::GPS],
                },
            ),
            (
                "gps,gal:testing",
                NavFilter {
                    filter: NavFilterType::Testing,
                    constellations: vec![Constellation::GPS, Constellation::Galileo],
                },
            ),
        ] {
            let parsed = NavFilter::from_str(value)
                .unwrap_or_else(|e| panic!("Failed to parse from \"{}\": {}", value, e));

            assert_eq!(parsed, expected);
        }
    }
}
