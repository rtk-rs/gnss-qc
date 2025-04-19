use crate::prelude::Rinex;

#[cfg(feature = "sp3")]
use crate::prelude::SP3;

#[derive(Clone)]
pub enum BlobData {
    /// [Rinex] content
    RINEX(Rinex),
    #[cfg(feature = "sp3")]
    /// [SP3] content
    SP3(SP3),
}

impl BlobData {
    /// Returns reference to underlying [Rinex] data
    pub fn as_rinex(&self) -> Option<&Rinex> {
        match self {
            Self::RINEX(r) => Some(r),
            #[cfg(feature = "sp3")]
            _ => None,
        }
    }

    /// Returns mutable reference to underlying [Rinex] data
    pub fn as_mut_rinex(&mut self) -> Option<&mut Rinex> {
        match self {
            Self::RINEX(r) => Some(r),
            #[cfg(feature = "sp3")]
            _ => None,
        }
    }
}

/// Returns reference to inner SP3 data.
#[cfg(feature = "sp3")]
#[cfg_attr(docsrs, doc(cfg(feature = "sp3")))]
impl BlobData {
    /// Returns reference to underlying [SP3] data
    pub fn as_sp3(&self) -> Option<&SP3> {
        match self {
            Self::SP3(s) => Some(s),
            _ => None,
        }
    }

    /// Returns mutable reference to underlying [SP3] data
    pub fn as_mut_sp3(&mut self) -> Option<&mut SP3> {
        match self {
            Self::SP3(s) => Some(s),
            _ => None,
        }
    }
}
