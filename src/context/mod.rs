//! GNSS processing context definition.
use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
};

use crate::prelude::{Rinex, TimeScale};

use qc_traits::Merge;

pub mod time;

pub(crate) mod blob;
use blob::BlobData;

#[cfg(feature = "flate2")]
#[cfg_attr(docsrs, doc(cfg(feature = "flate2")))]
mod flate2;

#[cfg(feature = "sp3")]
#[cfg_attr(docsrs, doc(cfg(feature = "sp3")))]
mod sp3;

#[cfg(feature = "navigation")]
#[cfg_attr(docsrs, doc(cfg(feature = "navigation")))]
mod navigation;

use qc_traits::{Filter, Preprocessing, Repair, RepairTrait};

use crate::{error::Error, prelude::ProductType};

#[cfg(feature = "navigation")]
use crate::prelude::{Almanac, Frame};

/// [QcContext] is a general structure capable to store most common
/// GNSS data. It is dedicated to post processing workflows,
/// precise timing or atmosphere analysis.
#[derive(Clone)]
pub struct QcContext {
    /// Files merged into this [QcContext]
    pub(crate) files: HashMap<ProductType, Vec<PathBuf>>,

    /// Context blob created by merging each members of each category
    pub(crate) blob: HashMap<ProductType, BlobData>,

    #[cfg(feature = "navigation")]
    #[cfg_attr(docsrs, doc(cfg(feature = "navigation")))]
    /// Latest [Almanac]
    pub almanac: Almanac,

    #[cfg(feature = "navigation")]
    #[cfg_attr(docsrs, doc(cfg(feature = "navigation")))]
    /// ECEF [Frame]
    pub earth_cef: Frame,
}

impl QcContext {
    /// Creates a new [QcContext] for GNSS post processing.
    ///
    /// For people interested in Post Processed navigation:
    /// - if the library was compiled with "embed_ephem" option, you are good
    /// to go for high precision navigation. Otherwise, this method will require
    /// that a navigation cache is created and requires internet access on first deployment.
    /// - for people targeting ultra high navigation precision, you should
    /// use the JPL BPC cache and keep it up to date, by using [Self::with_jpl_update],
    /// which requires internet access at all times.
    ///
    /// ```
    /// use gnss_qc::prelude::{QcContext, TimeScale};
    ///
    /// // create a new (empty) context
    /// let mut context = QcContext::new();
    ///
    /// // load some data
    /// context.load_rinex_file("data/OBS/V2/AJAC3550.21O")
    ///     .unwrap();
    ///
    /// // do something
    /// assert_eq!(context.timescale(), Some(TimeScale::GPST));
    /// ```
    pub fn new() -> Self {
        #[cfg(feature = "navigation")]
        let (almanac, earth_cef) = Self::default_almanac_frame();

        Self {
            files: Default::default(),
            blob: Default::default(),
            #[cfg(feature = "navigation")]
            almanac,
            #[cfg(feature = "navigation")]
            earth_cef,
        }
    }

    /// Returns "main" [TimeScale] for current [QcContext].
    ///
    /// In case measurements where provided, they will always prevail:
    /// ```
    /// use gnss_qc::prelude::{QcContext, TimeScale};
    ///
    /// // create a new (empty) context
    /// let mut context = QcContext::new();
    ///
    /// // load some data
    /// context.load_rinex_file("data/OBS/V2/AJAC3550.21O")
    ///     .unwrap();
    ///
    /// context.load_rinex_file("data/NAV/V2/amel0010.21g")
    ///     .unwrap();
    ///
    /// assert_eq!(context.timescale(), Some(TimeScale::GPST));
    /// ```
    ///
    /// SP3 files have unambiguous timescale definition as well.
    /// So they will prevail as long as RINEX measurements were not provided:
    ///
    /// ```
    /// use gnss_qc::prelude::{QcContext, TimeScale};
    ///
    /// // create a new (empty) context
    /// let mut context = QcContext::new();
    ///
    /// // load some data
    /// context.load_gzip_sp3_file("data/SP3/D/COD0MGXFIN_20230500000_01D_05M_ORB.SP3.gz")
    ///     .unwrap();
    ///
    /// assert_eq!(context.timescale(), Some(TimeScale::GPST));
    /// ```
    pub fn timescale(&self) -> Option<TimeScale> {
        if let Some(obs) = self.observation() {
            let first = obs.first_epoch()?;
            Some(first.time_scale)
        } else if let Some(dor) = self.doris() {
            let first = dor.first_epoch()?;
            Some(first.time_scale)
        } else if let Some(clk) = self.clock() {
            let first = clk.first_epoch()?;
            Some(first.time_scale)
        } else if self.meteo().is_some() {
            Some(TimeScale::UTC)
        } else if self.ionex().is_some() {
            Some(TimeScale::UTC)
        } else {
            #[cfg(feature = "sp3")]
            if let Some(sp3) = self.sp3() {
                return Some(sp3.header.timescale);
            }

            None
        }
    }

    /// Returns path to File considered as Primary product in this Context.
    /// When a unique file had been loaded, it is obviously considered Primary.
    pub fn primary_path(&self) -> Option<&PathBuf> {
        /*
         * Order is important: determines what format are prioritized
         * in the "primary" determination
         */
        for product in [
            ProductType::Observation,
            ProductType::DORIS,
            ProductType::BroadcastNavigation,
            ProductType::MeteoObservation,
            ProductType::IONEX,
            ProductType::ANTEX,
            ProductType::HighPrecisionClock,
            #[cfg(feature = "sp3")]
            ProductType::HighPrecisionOrbit,
        ] {
            if let Some(paths) = self.files(product) {
                /*
                 * Returns Fist file loaded in this category
                 */
                return paths.first();
            }
        }
        None
    }

    /// Returns name of this context.
    /// Context is named after the file considered as Primary, see [Self::primary_path].
    /// If no files were previously loaded, simply returns "Undefined".
    pub fn name(&self) -> String {
        if let Some(path) = self.primary_path() {
            path.file_name()
                .unwrap_or(OsStr::new("Undefined"))
                .to_string_lossy()
                // removes possible .crx ; .gz extensions
                .split('.')
                .next()
                .unwrap_or("Undefined")
                .to_string()
        } else {
            "Undefined".to_string()
        }
    }

    /// Returns reference to files loaded in given category
    pub fn files(&self, product: ProductType) -> Option<&Vec<PathBuf>> {
        self.files
            .iter()
            .filter_map(|(prod_type, paths)| {
                if *prod_type == product {
                    Some(paths)
                } else {
                    None
                }
            })
            .reduce(|k, _| k)
    }

    /// Returns mutable reference to files loaded in given category
    pub fn files_mut(&mut self, product: ProductType) -> Option<&Vec<PathBuf>> {
        self.files
            .iter()
            .filter_map(|(prod_type, paths)| {
                if *prod_type == product {
                    Some(paths)
                } else {
                    None
                }
            })
            .reduce(|k, _| k)
    }

    /// Returns reference to inner data of given category
    pub(crate) fn data(&self, product: ProductType) -> Option<&BlobData> {
        self.blob
            .iter()
            .filter_map(|(prod_type, data)| {
                if *prod_type == product {
                    Some(data)
                } else {
                    None
                }
            })
            .reduce(|k, _| k)
    }

    /// Returns mutable reference to inner data of given category
    pub(crate) fn data_mut(&mut self, product: ProductType) -> Option<&mut BlobData> {
        self.blob
            .iter_mut()
            .filter_map(|(prod_type, data)| {
                if *prod_type == product {
                    Some(data)
                } else {
                    None
                }
            })
            .reduce(move |k, _| k)
    }

    /// Returns reference to inner RINEX data of given category
    /// ```
    /// use gnss_qc::prelude::{QcContext, ProductType};
    ///
    /// // create a new (empty) context
    /// let mut context = QcContext::new();
    ///
    /// // load some data
    /// context.load_rinex_file("data/OBS/V2/AJAC3550.21O")
    ///     .unwrap();
    ///
    /// // retrieve
    /// let rinex = context.rinex(ProductType::Observation)
    ///     .unwrap();
    ///
    /// // do something
    /// assert!(rinex.is_observation_rinex());
    /// ```
    pub fn rinex(&self, product: ProductType) -> Option<&Rinex> {
        self.data(product)?.as_rinex()
    }

    /// Returns mutable reference to inner RINEX data of given category
    pub fn rinex_mut(&mut self, product: ProductType) -> Option<&mut Rinex> {
        self.data_mut(product)?.as_mut_rinex()
    }

    /// Returns reference to inner [ProductType::Observation] data
    pub fn observation(&self) -> Option<&Rinex> {
        self.data(ProductType::Observation)?.as_rinex()
    }

    /// Returns reference to inner [ProductType::DORIS] RINEX data
    pub fn doris(&self) -> Option<&Rinex> {
        self.data(ProductType::DORIS)?.as_rinex()
    }

    /// Returns reference to inner [ProductType::BroadcastNavigation] data
    pub fn brdc_navigation(&self) -> Option<&Rinex> {
        self.data(ProductType::BroadcastNavigation)?.as_rinex()
    }

    /// Returns reference to inner [ProductType::Meteo] data
    pub fn meteo(&self) -> Option<&Rinex> {
        self.data(ProductType::MeteoObservation)?.as_rinex()
    }

    /// Returns reference to inner [ProductType::HighPrecisionClock] data
    pub fn clock(&self) -> Option<&Rinex> {
        self.data(ProductType::HighPrecisionClock)?.as_rinex()
    }

    /// Returns reference to inner [ProductType::ANTEX] data
    pub fn antex(&self) -> Option<&Rinex> {
        self.data(ProductType::ANTEX)?.as_rinex()
    }

    /// Returns reference to inner [ProductType::IONEX] data
    pub fn ionex(&self) -> Option<&Rinex> {
        self.data(ProductType::IONEX)?.as_rinex()
    }

    /// Returns mutable reference to inner [ProductType::Observation] data
    pub fn observation_mut(&mut self) -> Option<&mut Rinex> {
        self.data_mut(ProductType::Observation)?.as_mut_rinex()
    }

    /// Returns mutable reference to inner [ProductType::DORIS] RINEX data
    pub fn doris_mut(&mut self) -> Option<&mut Rinex> {
        self.data_mut(ProductType::DORIS)?.as_mut_rinex()
    }

    /// Returns mutable reference to inner [ProductType::Observation] data
    pub fn brdc_navigation_mut(&mut self) -> Option<&mut Rinex> {
        self.data_mut(ProductType::BroadcastNavigation)?
            .as_mut_rinex()
    }

    /// Returns reference to inner [ProductType::Meteo] data
    pub fn meteo_mut(&mut self) -> Option<&mut Rinex> {
        self.data_mut(ProductType::MeteoObservation)?.as_mut_rinex()
    }

    /// Returns mutable reference to inner [ProductType::HighPrecisionClock] data
    pub fn clock_mut(&mut self) -> Option<&mut Rinex> {
        self.data_mut(ProductType::HighPrecisionClock)?
            .as_mut_rinex()
    }

    /// Returns mutable reference to inner [ProductType::ANTEX] data
    pub fn antex_mut(&mut self) -> Option<&mut Rinex> {
        self.data_mut(ProductType::ANTEX)?.as_mut_rinex()
    }

    /// Returns mutable reference to inner [ProductType::IONEX] data
    pub fn ionex_mut(&mut self) -> Option<&mut Rinex> {
        self.data_mut(ProductType::IONEX)?.as_mut_rinex()
    }

    /// Returns true if [ProductType::Observation] are present in Self
    pub fn has_observation(&self) -> bool {
        self.observation().is_some()
    }

    /// Returns true if [ProductType::BroadcastNavigation] are present in Self
    pub fn has_brdc_navigation(&self) -> bool {
        self.brdc_navigation().is_some()
    }

    /// Returns true if at least one [ProductType::DORIS] file is present
    pub fn has_doris(&self) -> bool {
        self.doris().is_some()
    }

    /// Returns true if [ProductType::MeteoObservation] are present in Self
    pub fn has_meteo(&self) -> bool {
        self.meteo().is_some()
    }

    /// Load a readable [Rinex] file into this [QcContext].
    pub fn load_rinex_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
        let rinex = Rinex::from_file(&path)?;
        self.load_rinex(path, rinex)
    }

    /// Load a single [Rinex] file into this [QcContext].
    /// File revision must be supported and must be correctly formatted
    /// for this operation to be effective.
    pub fn load_rinex<P: AsRef<Path>>(&mut self, path: P, rinex: Rinex) -> Result<(), Error> {
        let prod_type = ProductType::from(rinex.header.rinex_type);

        let path_buf = path.as_ref().to_path_buf();

        // extend context blob
        if let Some(paths) = self
            .files
            .iter_mut()
            .filter_map(|(prod, files)| {
                if *prod == prod_type {
                    Some(files)
                } else {
                    None
                }
            })
            .reduce(|k, _| k)
        {
            if let Some(inner) = self.blob.get_mut(&prod_type).and_then(|k| k.as_mut_rinex()) {
                inner.merge_mut(&rinex)?;
                paths.push(path_buf);
            }
        } else {
            self.blob.insert(prod_type, BlobData::RINEX(rinex));
            self.files.insert(prod_type, vec![path_buf]);
        }

        Ok(())
    }

    /// True if current [QcContext] is compatible with basic post processed navigation.
    /// It does not mean you can actually perform post processed navigation, you need the "navigation"
    /// feature for that.
    pub fn is_navigation_compatible(&self) -> bool {
        self.observation().is_some() && self.brdc_navigation().is_some()
    }

    /// Returns true if provided Input products allow Ionosphere bias
    /// model optimization
    pub fn iono_bias_model_optimization(&self) -> bool {
        self.ionex().is_some() // TODO: BRDC V3 or V4
    }

    /// Returns true if provided Input products allow Troposphere bias
    /// model optimization
    pub fn tropo_bias_model_optimization(&self) -> bool {
        self.has_meteo()
    }

    /// Apply preprocessing filter algorithm to mutable [Self].
    /// Filter will apply to all data contained in the context.
    pub fn filter_mut(&mut self, filter: &Filter) {
        if let Some(data) = self.observation_mut() {
            data.filter_mut(filter);
        }
        if let Some(data) = self.brdc_navigation_mut() {
            data.filter_mut(filter);
        }
        if let Some(data) = self.doris_mut() {
            data.filter_mut(filter);
        }
        if let Some(data) = self.meteo_mut() {
            data.filter_mut(filter);
        }
        if let Some(data) = self.clock_mut() {
            data.filter_mut(filter);
        }
        if let Some(data) = self.ionex_mut() {
            data.filter_mut(filter);
        }

        #[cfg(feature = "sp3")]
        if let Some(data) = self.sp3_mut() {
            data.filter_mut(filter);
        }
    }

    /// Fix given [Repair] condition
    pub fn repair_mut(&mut self, r: Repair) {
        if let Some(rinex) = self.observation_mut() {
            rinex.repair_mut(r);
        }
    }

    /// True if current [QcContext] is compatible with CPP positioning method
    /// <https://docs.rs/gnss-rtk/latest/gnss_rtk/prelude/enum.Method.html#variant.CodePPP>.
    /// This does not mean you can deploy a navigation solver, because that requires
    /// the "navigation" create feature.
    pub fn is_cpp_navigation_compatible(&self) -> bool {
        // TODO: improve: only PR
        if let Some(obs) = self.observation() {
            obs.carrier_iter().count() > 1
        } else {
            false
        }
    }

    /// Returns True if current [QcContext] is compatible with PPP positioning method
    /// <https://docs.rs/gnss-rtk/latest/gnss_rtk/prelude/enum.Method.html#variant.PPP>.
    /// This does not mean you can deploy a navigation solver, because that requires
    /// the "navigation" create feature.
    pub fn is_ppp_navigation_compatible(&self) -> bool {
        // TODO: check PH as well
        self.is_cpp_navigation_compatible()
    }

    #[cfg(not(feature = "sp3"))]
    /// SP3 is required for 100% PPP compatibility
    pub fn is_ppp_ultra_navigation_compatible(&self) -> bool {
        false
    }
}

impl std::fmt::Debug for QcContext {
    /// Debug formatting, prints all loaded files per Product category.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Primary: \"{}\"", self.name())?;
        for product in [
            ProductType::Observation,
            ProductType::BroadcastNavigation,
            ProductType::MeteoObservation,
            ProductType::HighPrecisionClock,
            ProductType::IONEX,
            ProductType::ANTEX,
            #[cfg(feature = "sp3")]
            ProductType::HighPrecisionOrbit,
        ] {
            if let Some(files) = self.files(product) {
                write!(f, "\n{}: ", product)?;
                write!(f, "{:?}", files,)?;
            }
        }
        Ok(())
    }
}
