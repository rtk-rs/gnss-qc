use crate::{
    context::BlobData,
    error::Error,
    prelude::{ProductType, QcContext, SP3},
};

use qc_traits::Merge;

use std::path::Path;

impl QcContext {
    /// Add this [SP3] into current [QcContext].
    /// File revision must be supported and must be correctly formatted
    /// for this operation to be effective.
    pub fn load_sp3<P: AsRef<Path>>(&mut self, path: P, sp3: SP3) -> Result<(), Error> {
        let prod_type = ProductType::HighPrecisionOrbit;

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
            if let Some(inner) = self.blob.get_mut(&prod_type).and_then(|k| k.as_mut_sp3()) {
                inner.merge_mut(&sp3)?;
                paths.push(path_buf);
            }
        } else {
            self.blob.insert(prod_type, BlobData::SP3(sp3));
            self.files.insert(prod_type, vec![path_buf]);
        }
        Ok(())
    }

    /// Load readable [SP3] file into this [QcContext].
    pub fn load_sp3_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
        let sp3 = SP3::from_file(&path)?;
        self.load_sp3(path, sp3)
    }

    /// Returns true if [ProductType::HighPrecisionOrbit] are present in current [QcContext]
    pub fn has_sp3(&self) -> bool {
        self.sp3().is_some()
    }

    /// Returns true if any [SP3] previously loaded came with clock information.
    pub fn sp3_has_clock(&self) -> bool {
        if let Some(sp3) = self.sp3() {
            sp3.has_satellite_clock_offset()
        } else {
            false
        }
    }

    /// Returns reference to inner SP3 data
    pub fn sp3(&self) -> Option<&SP3> {
        self.data(ProductType::HighPrecisionOrbit)?.as_sp3()
    }

    /// Returns mutable reference to inner [ProductType::HighPrecisionOrbit] data
    pub fn sp3_mut(&mut self) -> Option<&mut SP3> {
        self.data_mut(ProductType::HighPrecisionOrbit)?.as_mut_sp3()
    }

    pub fn is_ppp_ultra_navigation_compatible(&self) -> bool {
        // TODO: improve
        //      verify clock().ts and obs().ts do match
        //      and have common time frame
        self.clock().is_some() && self.sp3_has_clock() && self.is_cpp_navigation_compatible()
    }
}
