use crate::prelude::{QcContext, TimeScale};

use qc_traits::{GnssAbsoluteTime, TimePolynomial, Timeshift};

impl QcContext {
    /// Form a [GnssAbsoluteTime] solver from this [QcContext],
    /// used to allow transposition into other [TimeScale]s.
    /// This requires navigation both feature and compatibility to truly be effective.
    pub fn gnss_absolute_time_solver(&self) -> GnssAbsoluteTime {
        let mut polynomials = Vec::<TimePolynomial>::new();

        if let Some(brdc) = self.brdc_navigation() {
            if let Some(brdc) = &brdc.header.nav {
                for time_offset in brdc.time_offsets.iter() {
                    polynomials.push(TimePolynomial::from_reference_time_of_week_nanos(
                        time_offset.lhs,
                        time_offset.t_ref.0,
                        time_offset.t_ref.1,
                        time_offset.rhs,
                        time_offset.polynomials,
                    ));
                }
            }
        }

        GnssAbsoluteTime::new(&polynomials)
    }

    /// Perform precise transposition of each individual components (input products) contained in current [QcContext]
    /// to desired [TimeScale]. NB: transposition might not be feasible for some components, therefore
    /// you should double check the newly obtained [QcContext].
    ///
    /// This may apply to your [SP3] products, if feature is activated.
    ///
    /// Example (1): precise RINEX transpositions
    ///
    /// Example (2): RINEX + SP3 PPP transposition
    ///
    pub fn timescale_transposition(&self, target: TimeScale) -> Self {
        let mut s = self.clone();
        s.timescale_transposition_mut(target);
        s
    }

    pub fn timescale_transposition_mut(&mut self, target: TimeScale) {
        let solver = self.gnss_absolute_time_solver();

        if let Some(observations) = self.observation_mut() {
            observations.timeshift_mut(&solver, target);
        }

        #[cfg(feature = "sp3")]
        if let Some(sp3) = self.sp3_mut() {
            sp3.timeshift_mut(&solver, target);
        }
    }
}
