use crate::prelude::{ProductType, QcContext, TimeScale};

use qc_traits::{Merge, TimeCorrectionError, TimeCorrectionsDB, Timeshift};

impl QcContext {
    /// Collect a [TimeCorrectionDB] from this [QcContext], that you can then
    /// use for precise temporal correction. The database will contain
    /// all time corrections available and described by this dataset.
    /// This requires both navigation feature and navigation compatibility to truly be effective.
    pub fn time_corrections_database(&self) -> TimeCorrectionsDB {
        let mut db = TimeCorrectionsDB::default();

        if let Some(brdc) = self.brdc_navigation() {
            if let Some(nav_db) = brdc.time_corrections_database() {
                db.merge_mut(&nav_db).unwrap(); // infaillible
            }
        }

        db
    }

    /// Infaillible transposition of the temporal products to desired [TimeScale].
    /// This only applies to the following products:
    /// - Observation RINEX
    /// - SP3
    /// ```
    /// use gnss_qc::prelude::{QcContext, TimeScale};
    ///
    /// let mut context = QcContext::new();
    ///
    /// // GPST observations
    /// context.load_gzip_rinex_file("data/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz")
    ///     .unwrap();
    ///
    /// // GPST sP3
    /// context.load_gzip_sp3_file("data/SP3/D/example.txt")
    ///     .unwrap();
    ///
    /// // convert both to GST
    /// context.timescale_transposition_mut(TimeScale::GST);
    /// ```
    pub fn timescale_transposition_mut(&mut self, timescale: TimeScale) {
        for (product_type, data) in self.blob.iter_mut() {
            match product_type {
                ProductType::Observation => {
                    let rinex = data.as_mut_rinex().unwrap();
                    rinex.timeshift_mut(timescale);
                }
                #[cfg(feature = "sp3")]
                ProductType::HighPrecisionOrbit => {
                    let sp3 = data.as_mut_sp3().unwrap();
                    sp3.timeshift_mut(timescale);
                }
                _ => {}
            }
        }
    }

    /// Precise temporal transposition of each individual products contained in current [QcContext].
    /// NB: transposition might not be feasible for some components, therefore
    /// you should double check the newly obtained [QcContext].
    ///
    /// This may apply to your [SP3] products, if feature is activated.
    ///
    /// Example (1): precise RINEX transpositions
    /// ```
    /// use gnss_qc::prelude::{QcContext, TimeScale};
    ///
    /// let mut context = QcContext::new();
    ///
    /// // GPST observations
    /// context.load_gzip_rinex_file("data/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz")
    ///     .unwrap();
    ///
    /// // Transposition attempt
    /// let transposed = context.timescale_transposition(TimeScale::GST);
    /// let transposed_obs = transposed.observation().unwrap();
    ///
    /// // For this to work, Observations are not enough.
    /// for t in transposed_obs.epoch_iter() {
    ///     assert_eq!(t.time_scale, TimeScale::GPST);
    /// }
    ///
    /// // You need to stack NAV RINEX for that day as well
    /// context.load_gzip_rinex_file("data/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz")
    ///     .unwrap();
    ///
    /// let transposed = context.timescale_transposition(TimeScale::GST);
    /// let transposed_obs = transposed.observation().unwrap();
    ///
    /// // For this to work, Observations are not enough.
    /// for t in transposed_obs.epoch_iter() {
    ///     assert_eq!(t.time_scale, TimeScale::GST);
    /// }
    /// ```
    ///
    /// Example (2): SP3 transposition.
    /// SP3 are totally valid in any GNSS timescale, you can use this framework
    /// to reformat as desired !
    ///
    pub fn precise_time_correction_mut(
        &mut self,
        db: TimeCorrectionsDB,
        timescale: TimeScale,
    ) -> Result<(), TimeCorrectionError> {
        if let Some(observations) = self.observation_mut() {
            observations.precise_correction_mut(&db, timescale)?;
        }

        #[cfg(feature = "sp3")]
        if let Some(sp3) = self.sp3_mut() {
            sp3.precise_correction_mut(&db, timescale)?;
        }

        Ok(())
    }
}
