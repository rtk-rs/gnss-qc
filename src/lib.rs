#![doc(html_logo_url = "https://raw.githubusercontent.com/rtk-rs/.github/master/logos/logo2.jpg")]
#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

/*
 * GNSS-Qc is part of the rtk-rs framework.
 * Authors: Guillaume W. Bres <guillaume.bressaix@gmail.com> et al.
 * (cf. https://github.com/rtk-rs/gnss-qc/graphs/contributors)
 * This framework is shipped under Mozilla Public V2 license.
 *
 * Documentation:
 * - https://github.com/rtk-rs/gnss-qc
 * - https://github.com/rtk-rs/rinex
 * - https://github.com/rtk-rs/sp3
 */

#[cfg(feature = "navigation")]
#[macro_use]
extern crate log;

extern crate gnss_qc_traits as qc_traits;
extern crate gnss_rs as gnss;

mod cfg;
mod context;
mod product;
mod report;

#[cfg(feature = "navigation")]
#[cfg_attr(docsrs, doc(cfg(feature = "navigation")))]
mod navigation;

pub mod error;
pub mod plot;

#[cfg(test)]
mod tests;

pub mod prelude {
    pub use crate::{
        cfg::{QcConfig, QcReportType},
        context::QcContext,
        error::Error,
        product::ProductType,
        report::{QcExtraPage, QcReport},
    };

    pub use gnss::prelude::{Constellation, COSPAR, SV};
    pub use hifitime::prelude::{Duration, Epoch, TimeScale};

    #[cfg(feature = "navigation")]
    pub use crate::navigation::{NavFilter, NavFilterType, ReferenceEcefPosition};

    pub use crate::plot::{Marker, MarkerSymbol, Mode, Plot};

    pub use qc_traits::{
        Filter, FilterItem, MaskOperand, Preprocessing, Repair, RepairTrait, TimeCorrection,
        TimeCorrectionError, TimeCorrectionsDB, Timeshift,
    };

    pub use rinex::prelude::{Error as RinexError, Rinex};

    #[cfg(feature = "navigation")]
    pub use anise::prelude::{Almanac, Frame, Orbit};

    #[cfg(feature = "sp3")]
    pub use sp3::prelude::{Error as SP3Error, SP3};

    pub use std::path::Path;

    pub use maud::{html, Markup, Render};
}
