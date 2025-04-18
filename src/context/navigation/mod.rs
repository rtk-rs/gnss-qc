/*
 * All Post Proecessed Navigation support & feature dependent stuff.
 *
 * Authors: Guillaume W. Bres <guillaume.bressaix@gmail.com> et al.
 * (cf. https://github.com/rtk-rs/gnss-qc/graphs/contributors)
 * This framework is shipped under Mozilla Public V2 license.
 *
 * Documentation:
 * - https://github.com/rtk-rs/gnss-qc
 * - https://github.com/rtk-rs/rinex
 * - https://github.com/rtk-rs/sp3
 */

use thiserror::Error;

use anise::{
    almanac::{
        metaload::{MetaAlmanacError, MetaFile},
        planetary::PlanetaryDataError,
    },
    constants::frames::{EARTH_ITRF93, EARTH_J2000},
    errors::AlmanacError,
    prelude::{Almanac, Frame, MetaAlmanac},
};

use crate::{
    navigation::{NavFilter, NavFilterType},
    prelude::{Constellation, QcContext},
};

#[derive(Debug, Error)]
pub enum NavigationError {
    #[error("almanac error: {0}")]
    Almanac(#[from] AlmanacError),
    #[error("meta error: {0}")]
    MetaAlmanac(#[from] MetaAlmanacError),
    #[error("planetary data error")]
    PlanetaryData(#[from] PlanetaryDataError),
}

impl QcContext {
    fn anise_de440s_bsp() -> MetaFile {
        MetaFile {
            crc32: Some(0x7286750a),
            uri: String::from("http://public-data.nyxspace.com/anise/de440s.bsp"),
        }
    }

    fn anise_pck11_pca() -> MetaFile {
        MetaFile {
            crc32: Some(0x8213b6e9),
            uri: String::from("http://public-data.nyxspace.com/anise/v0.5/pck11.pca"),
        }
    }

    fn anise_jpl_bpc() -> MetaFile {
        MetaFile {
            crc32: None,
            uri:
                "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/pck/earth_latest_high_prec.bpc"
                    .to_string(),
        }
    }

    /// This [MetaAlmanac] solely relies on the nyx-space servers
    fn default_meta_almanac() -> MetaAlmanac {
        MetaAlmanac {
            files: vec![Self::anise_pck11_pca(), Self::anise_de440s_bsp()],
        }
    }

    /// This [MetaAlmanac] solely relies on the nyx-space servers
    fn high_precision_meta_almanac() -> MetaAlmanac {
        MetaAlmanac {
            files: vec![
                Self::anise_pck11_pca(),
                Self::anise_de440s_bsp(),
                Self::anise_jpl_bpc(),
            ],
        }
    }

    /// Obtains [Almanac] + ECEF [Frame] definition from ANISE database
    pub(crate) fn default_almanac_frame() -> (Almanac, Frame) {
        let mut meta = Self::default_meta_almanac();

        let almanac = match meta.process(false) {
            Ok(almanac) => almanac,
            Err(e) => {
                error!("anise error: {}", e);
                Almanac::default()
            }
        };

        let frame = almanac
            .frame_from_uid(EARTH_J2000)
            .unwrap_or_else(|e| panic!("anise internal error: {}", e));

        (almanac, frame)
    }

    /// Upgrade this [QcContext] for ultra high precision navigation.
    pub fn with_jpl_bpc(&self) -> Result<(), NavigationError> {
        let mut s = self.clone();

        let mut meta = Self::high_precision_meta_almanac();
        let almanac = meta.process(true)?;

        s.almanac = almanac;

        let mut meta = Self::default_meta_almanac();
        let almanac = meta.process(true)?;

        let frame = almanac.frame_from_uid(EARTH_ITRF93)?;
        s.earth_cef = frame;

        Ok(())
    }

    /// Applies complex [NavFilter] to mutable [QcContext].
    pub fn nav_filter_mut(&mut self, filter: &NavFilter) {
        // apply nav conditions
        if let Some(brdc) = self.brdc_navigation_mut() {
            let any_constellation = filter.constellations.is_empty();
            let broad_sbas = filter.constellations.contains(&Constellation::SBAS);

            let brdc_rec = brdc.record.as_mut_nav().unwrap();

            brdc_rec.retain(|k, data| {
                if let Some(eph) = data.as_ephemeris() {
                    match filter.filter {
                        NavFilterType::Healthy => {
                            if k.sv.constellation.is_sbas() && broad_sbas {
                                eph.sv_healthy()
                            } else {
                                if any_constellation {
                                    eph.sv_healthy()
                                } else {
                                    if filter.constellations.contains(&k.sv.constellation) {
                                        eph.sv_healthy()
                                    } else {
                                        true
                                    }
                                }
                            }
                        }
                        NavFilterType::Testing => {
                            if k.sv.constellation.is_sbas() && broad_sbas {
                                eph.sv_in_testing()
                            } else {
                                if any_constellation {
                                    eph.sv_in_testing()
                                } else {
                                    if filter.constellations.contains(&k.sv.constellation) {
                                        eph.sv_in_testing()
                                    } else {
                                        true
                                    }
                                }
                            }
                        }
                        NavFilterType::Unhealthy => {
                            if k.sv.constellation.is_sbas() && broad_sbas {
                                !eph.sv_healthy()
                            } else {
                                if any_constellation {
                                    !eph.sv_healthy()
                                } else {
                                    if filter.constellations.contains(&k.sv.constellation) {
                                        !eph.sv_healthy()
                                    } else {
                                        true
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // preserves other frames
                    true
                }
            });
        }
    }

    /// True if current [QcContext] is compatible with CPP positioning method
    /// <https://docs.rs/gnss-rtk/latest/gnss_rtk/prelude/enum.Method.html#variant.CodePPP>
    pub fn is_cpp_navigation_compatible(&self) -> bool {
        // TODO: improve: only PR
        if let Some(obs) = self.observation() {
            obs.carrier_iter().count() > 1
        } else {
            false
        }
    }

    /// Returns True if current [QcContext] is compatible with PPP positioning method
    /// <https://docs.rs/gnss-rtk/latest/gnss_rtk/prelude/enum.Method.html#variant.PPP>
    pub fn is_ppp_navigation_compatible(&self) -> bool {
        // TODO: check PH as well
        self.is_cpp_navigation_compatible()
    }

    #[cfg(not(feature = "sp3"))]
    /// SP3 is required for 100% PPP compatibility
    pub fn ppp_ultra_navigation_compatible(&self) -> bool {
        false
    }

    #[cfg(feature = "sp3")]
    pub fn is_ppp_ultra_navigation_compatible(&self) -> bool {
        // TODO: improve
        //      verify clock().ts and obs().ts do match
        //      and have common time frame
        self.clock().is_some() && self.sp3_has_clock() && self.is_cpp_navigation_compatible()
    }
}
