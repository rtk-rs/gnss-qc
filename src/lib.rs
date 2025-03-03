#![doc(html_logo_url = "https://raw.githubusercontent.com/rtk-rs/.github/master/logos/logo2.jpg")]
#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[macro_use]
extern crate log;

extern crate gnss_qc_traits as qc_traits;
extern crate gnss_rs as gnss;

mod cfg;
mod context;
mod product;
mod report;

pub mod error;
pub mod plot;

pub mod prelude {
    pub use crate::{
        cfg::{QcConfig, QcReportType},
        context::QcContext,
        error::Error,
        product::ProductType,
        report::{QcExtraPage, QcReport},
    };
    // Pub re-export
    pub use crate::plot::{Marker, MarkerSymbol, Mode, Plot};
    pub use maud::{html, Markup, Render};
    pub use qc_traits::{Filter, Preprocessing, Repair, RepairTrait};
    pub use rinex::prelude::nav::Almanac;
    pub use rinex::prelude::{Error as RinexError, Rinex};
    #[cfg(feature = "sp3")]
    pub use sp3::prelude::{Error as SP3Error, SP3};
    pub use std::path::Path;
}
