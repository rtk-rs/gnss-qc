use maud::{html, Markup, Render};
use thiserror::Error;

use serde::{Deserialize, Serialize};

/// Configuration Error
#[derive(Debug, Clone, Error)]
pub enum Error {
    #[error("invalid report type")]
    InvalidReportType,
}

use std::fmt::Display;
use std::str::FromStr;

/// [QcReportType]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QcReportType {
    /// In [Summary] mode, only the summary section
    /// of the report is to be generated. It is the lightest
    /// form we can generate.
    Summary,
    /// In [Full] mode, we generate the [CombinedReport] as well,
    /// which results from the consideration of all input [ProductType]s
    /// at the same time.
    #[default]
    Full,
}

impl FromStr for QcReportType {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "sum" | "summ" | "summary" => Ok(Self::Summary),
            "full" => Ok(Self::Full),
            _ => Err(Error::InvalidReportType),
        }
    }
}

impl Display for QcReportType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Full => f.write_str("Full"),
            Self::Summary => f.write_str("Summary"),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct QcConfig {
    #[serde(default)]
    pub report: QcReportType,

    #[cfg(feature = "navigation")]
    #[cfg_attr(docsrs, doc(cfg(feature = "navigation")))]
    #[serde(default)]
    pub user_rx_ecef: Option<(f64, f64, f64)>,
}

impl QcConfig {
    /// Define a new prefered [QcReportType].
    pub fn set_report_type(&mut self, report_type: QcReportType) {
        self.report = report_type;
    }

    /// Update the user defined RX position ECEF coordinates
    #[cfg(feature = "navigation")]
    #[cfg_attr(docsrs, doc(cfg(feature = "navigation")))]
    pub fn set_reference_rx_orbit(&mut self, ecef_m: (f64, f64, f64)) {
        self.user_rx_ecef = Some(ecef_m);
    }

    /// Build a [QcConfig] with updated [QcReportType] preference.
    pub fn with_report_type(&self, report_type: QcReportType) -> Self {
        let mut s = self.clone();
        s.report = report_type;
        s
    }

    /// Build a [QcConfig] with updated user defined RX position as ECEF coordinates.
    #[cfg(feature = "navigation")]
    #[cfg_attr(docsrs, doc(cfg(feature = "navigation")))]
    pub fn with_user_rx_position_ecef(&self, ecef_m: (f64, f64, f64)) -> Self {
        let mut s = self.clone();
        s.user_rx_ecef = Some(ecef_m);
        s
    }
}

impl Render for QcConfig {
    fn render(&self) -> Markup {
        html! {
            tr {
                td {
                    "Report"
                }
                td {
                    (self.report.to_string())
                }
            }
        }
    }
}
