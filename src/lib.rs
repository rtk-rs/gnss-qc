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

#[macro_use]
extern crate log;

extern crate gnss_qc_traits as qc_traits;
extern crate gnss_rs as gnss;

mod cfg;
mod context;
mod nav;
mod product;
mod report;

pub mod error;
pub mod plot;

pub mod prelude {
    pub use crate::{
        cfg::{QcConfig, QcReportType},
        context::QcContext,
        error::Error,
        nav::{NavFilter, NavFilterType},
        product::ProductType,
        report::{QcExtraPage, QcReport},
    };
    // Pub re-export
    pub use crate::plot::{Marker, MarkerSymbol, Mode, Plot};
    pub use maud::{html, Markup, Render};
    pub use qc_traits::{Filter, FilterItem, MaskOperand, Preprocessing, Repair, RepairTrait};
    pub use rinex::prelude::nav::Almanac;
    pub use rinex::prelude::{Error as RinexError, Rinex};
    #[cfg(feature = "sp3")]
    pub use sp3::prelude::{Error as SP3Error, SP3};
    pub use std::path::Path;
}
