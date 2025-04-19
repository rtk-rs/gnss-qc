use rinex::Rinex;

use crate::{error::Error, prelude::QcContext};

use std::path::Path;

#[cfg(feature = "sp3")]
use crate::prelude::SP3;

impl QcContext {
    /// Load a Gzip compressed RINEX file from readable [Path].
    pub fn load_gzip_rinex_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
        let rinex = Rinex::from_gzip_file(&path)?;
        self.load_rinex(path, rinex)
    }

    #[cfg(feature = "sp3")]
    #[cfg_attr(docsrs, doc(cfg(feature = "sp3")))]
    /// Load a Gzip compressed [SP3] file from readable [Path].
    pub fn load_gzip_sp3_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
        let sp3 = SP3::from_gzip_file(&path)?;
        self.load_sp3(path, sp3)
    }
}
