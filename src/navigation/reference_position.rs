use crate::prelude::{Epoch, Frame, Orbit};

pub struct ReferenceEcefPosition {
    /// Ecef coordinates in meters
    pub ecef_m: (f64, f64, f64),
}

impl ReferenceEcefPosition {
    /// Define new [ReferenceEcefPosition] from ECEF coordinates
    pub fn new(ecef_m: (f64, f64, f64)) -> Self {
        Self { ecef_m }
    }

    /// Create a new [ReferenceEcefPosition] from an [Orbit]
    pub fn from_orbit(orbit: &Orbit) -> Self {
        let posvel_m = orbit.to_cartesian_pos_vel() * 1.0E3;
        let ecef_m = (posvel_m[0], posvel_m[1], posvel_m[2]);
        Self { ecef_m }
    }

    /// Express this [ReferenceEcefPosition] as an [Orbit]
    pub fn to_orbit(&self, t: Epoch, frame: Frame) -> Orbit {
        let (x_km, y_km, z_km) = (
            self.ecef_m.0 * 1.0E-3,
            self.ecef_m.1 * 1.0E-3,
            self.ecef_m.2 * 1.0E-3,
        );

        Orbit::from_position(x_km, y_km, z_km, t, frame)
    }
}
