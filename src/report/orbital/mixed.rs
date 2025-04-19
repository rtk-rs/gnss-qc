use rinex::prelude::{nav::Orbit, Constellation, Epoch, Rinex, SV};
use std::collections::{BTreeMap, HashMap};

use qc_traits::{Filter, Preprocessing};

use crate::{
    plot::{MapboxStyle, MarkerSymbol, Mode},
    prelude::{html, Markup, Plot, QcContext, Render},
};

#[cfg(feature = "sp3")]
use sp3::prelude::SP3;

#[cfg(feature = "sp3")]
struct BrdcSp3Report {
    x_err_plot: Plot,
    y_err_plot: Plot,
    z_err_plot: Plot,
}

#[cfg(feature = "sp3")]
impl BrdcSp3Report {
    fn new(sp3: &SP3, brdc: &Rinex) -> Self {
        let mut errors = BTreeMap::<SV, Vec<(Epoch, f64, f64, f64)>>::new();
        for (t_sp3, sv_sp3, (sp3_x_km, sp3_y_km, sp3_z_km)) in sp3.satellites_position_km_iter() {
            if let Some(brdc_orb) = brdc.sv_orbit(sv_sp3, t_sp3) {
                let brdc_state = brdc_orb.to_cartesian_pos_vel();
                let (nav_x_km, nav_y_km, nav_z_km) = (brdc_state[0], brdc_state[1], brdc_state[2]);

                let (err_x_m, err_y_m, err_z_m) = (
                    (nav_x_km - sp3_x_km) * 1000.0,
                    (nav_y_km - sp3_y_km) * 1000.0,
                    (nav_z_km - sp3_z_km) * 1000.0,
                );

                if let Some(errors) = errors.get_mut(&sv_sp3) {
                    errors.push((t_sp3, err_x_m, err_y_m, err_z_m));
                } else {
                    errors.insert(sv_sp3, vec![(t_sp3, err_x_m, err_y_m, err_z_m)]);
                }
            }
        }
        Self {
            x_err_plot: {
                let mut plot = Plot::timedomain_plot(
                    "sp3_brdc_x_err",
                    "(BRDC - SP3) Position Errors",
                    "Error [m]",
                    true,
                );
                for (sv_index, (sv, errors)) in errors.iter().enumerate() {
                    let error_t = errors.iter().map(|(t, _, _, _)| *t).collect::<Vec<_>>();
                    let error_x = errors.iter().map(|(_, x, _, _)| *x).collect::<Vec<_>>();
                    let trace = Plot::timedomain_chart(
                        &sv.to_string(),
                        Mode::Markers,
                        MarkerSymbol::Diamond,
                        &error_t,
                        error_x,
                        sv_index < 4,
                    );
                    plot.add_trace(trace);
                }
                plot
            },
            y_err_plot: {
                let mut plot = Plot::timedomain_plot(
                    "sp3_brdc_y_err",
                    "(BRDC - SP3) Position Errors",
                    "Error [m]",
                    true,
                );
                for (sv_index, (sv, errors)) in errors.iter().enumerate() {
                    let error_t = errors.iter().map(|(t, _, _, _)| *t).collect::<Vec<_>>();
                    let error_y = errors.iter().map(|(_, _, y, _)| *y).collect::<Vec<_>>();
                    let trace = Plot::timedomain_chart(
                        &sv.to_string(),
                        Mode::Markers,
                        MarkerSymbol::Diamond,
                        &error_t,
                        error_y,
                        sv_index < 4,
                    );
                    plot.add_trace(trace);
                }
                plot
            },
            z_err_plot: {
                let mut plot = Plot::timedomain_plot(
                    "sp3_brdc_z_err",
                    "(BRDC - SP3) Position Errors",
                    "Error [m]",
                    true,
                );
                for (sv_index, (sv, errors)) in errors.iter().enumerate() {
                    let error_t = errors.iter().map(|(t, _, _, _)| *t).collect::<Vec<_>>();
                    let error_z = errors.iter().map(|(_, _, _, z)| *z).collect::<Vec<_>>();
                    let trace = Plot::timedomain_chart(
                        &sv.to_string(),
                        Mode::Markers,
                        MarkerSymbol::Diamond,
                        &error_t,
                        error_z,
                        sv_index < 4,
                    );
                    plot.add_trace(trace);
                }
                plot
            },
        }
    }
}

#[cfg(feature = "sp3")]
impl Render for BrdcSp3Report {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tr {
                        th class="is-info" {
                            "X errors"
                        }
                        td {
                            (self.x_err_plot.render())
                        }
                    }
                    tr {
                        th class="is-info" {
                            "Y errors"
                        }
                        td {
                            (self.y_err_plot.render())
                        }
                    }
                    tr {
                        th class="is-info" {
                            "Z errors"
                        }
                        td {
                            (self.z_err_plot.render())
                        }
                    }
                }
            }
        }
    }
}

pub struct OrbitReport {
    sky_plot: Plot,
    elev_plot: Plot,
    map_proj: Plot,
    // globe_proj: Plot,
    #[cfg(feature = "sp3")]
    brdc_sp3_err: HashMap<Constellation, BrdcSp3Report>,
}

impl OrbitReport {
    pub fn new(ctx: &QcContext, reference: Option<Orbit>) -> Self {
        let mut t_sp3 = BTreeMap::<SV, Vec<Epoch>>::new();
        let mut elev_sp3 = BTreeMap::<SV, Vec<f64>>::new();
        let mut azim_sp3 = BTreeMap::<SV, Vec<f64>>::new();
        let mut sp3_lat_ddeg = BTreeMap::<SV, Vec<f64>>::new();
        let mut sp3_long_ddeg = BTreeMap::<SV, Vec<f64>>::new();

        #[cfg(feature = "sp3")]
        if let Some(sp3) = ctx.sp3() {
            if let Some(rx_orbit) = reference {
                for (t, sp3_sv, sp3_orbit) in sp3.satellites_orbit_iter(ctx.earth_cef) {
                    if let Ok(az_el_range) = ctx
                        .almanac
                        .azimuth_elevation_range_sez(sp3_orbit, rx_orbit, None, None)
                    {
                        let (lat_ddeg, long_ddeg, _) = sp3_orbit
                            .latlongalt()
                            .unwrap_or_else(|e| panic!("laglongalt: physical error: {}", e));

                        if let Some(t_sp3) = t_sp3.get_mut(&sp3_sv) {
                            t_sp3.push(t);
                        } else {
                            t_sp3.insert(sp3_sv, vec![t]);
                        }

                        if let Some(e) = elev_sp3.get_mut(&sp3_sv) {
                            e.push(az_el_range.elevation_deg);
                        } else {
                            elev_sp3.insert(sp3_sv, vec![az_el_range.elevation_deg]);
                        }

                        if let Some(a) = azim_sp3.get_mut(&sp3_sv) {
                            a.push(az_el_range.azimuth_deg);
                        } else {
                            azim_sp3.insert(sp3_sv, vec![az_el_range.azimuth_deg]);
                        }

                        if let Some(lat) = sp3_lat_ddeg.get_mut(&sp3_sv) {
                            lat.push(lat_ddeg);
                        } else {
                            sp3_lat_ddeg.insert(sp3_sv, vec![lat_ddeg]);
                        }

                        if let Some(lon) = sp3_long_ddeg.get_mut(&sp3_sv) {
                            lon.push(long_ddeg);
                        } else {
                            sp3_long_ddeg.insert(sp3_sv, vec![long_ddeg]);
                        }
                    }
                }
            }
        }

        Self {
            sky_plot: {
                let mut plot = Plot::sky_plot("skyplot", "Sky plot", true);
                for (sv_index, (sv, epochs)) in t_sp3.iter().enumerate() {
                    let visible = sv_index < 4;
                    let elev_sp3 = elev_sp3.get(&sv).unwrap();
                    let azim_sp3 = azim_sp3.get(&sv).unwrap();
                    let trace = Plot::sky_trace(
                        &sv.to_string(),
                        epochs,
                        elev_sp3.to_vec(),
                        azim_sp3.to_vec(),
                        visible,
                    );
                    plot.add_trace(trace);
                }
                plot
            },
            elev_plot: {
                let mut elev_plot =
                    Plot::timedomain_plot("elev_plot", "Elevation", "Elevation [deg°]", true);
                for (sv_index, (sv, epochs)) in t_sp3.iter().enumerate() {
                    let elev = elev_sp3.get(&sv).unwrap();
                    let trace = Plot::timedomain_chart(
                        &sv.to_string(),
                        Mode::Markers,
                        MarkerSymbol::Diamond,
                        epochs,
                        elev.to_vec(),
                        sv_index < 4,
                    );
                    elev_plot.add_trace(trace);
                }
                elev_plot
            },
            map_proj: {
                let mut map_proj = Plot::world_map(
                    "map_proj",
                    "Map Projection",
                    MapboxStyle::OpenStreetMap,
                    (32.0, -40.0),
                    1,
                    true,
                );

                #[cfg(feature = "sp3")]
                if let Some(sp3) = ctx.sp3() {
                    for (sv_index, sv) in sp3.satellites_iter().enumerate() {
                        let lat_ddeg = sp3_lat_ddeg
                            .iter()
                            .filter_map(
                                |(svnn, v)| if *svnn == sv { Some(v.clone()) } else { None },
                            )
                            .collect::<Vec<_>>();

                        let long_ddeg = sp3_long_ddeg
                            .iter()
                            .filter_map(
                                |(svnn, v)| if *svnn == sv { Some(v.clone()) } else { None },
                            )
                            .collect::<Vec<_>>();

                        let map = Plot::mapbox(
                            lat_ddeg,
                            long_ddeg,
                            &sv.to_string(),
                            5,
                            MarkerSymbol::Circle,
                            None,
                            1.0,
                            sv_index < 2,
                        );

                        map_proj.add_trace(map);
                    }
                }
                map_proj
            },
            #[cfg(feature = "sp3")]
            brdc_sp3_err: {
                let mut reports = HashMap::<Constellation, BrdcSp3Report>::new();
                if let Some(sp3) = ctx.sp3() {
                    if let Some(nav) = ctx.brdc_navigation() {
                        for constellation in sp3.constellations_iter() {
                            if let Some(constellation) = nav
                                .constellations_iter()
                                .filter(|c| *c == constellation)
                                .reduce(|k, _| k)
                            {
                                let filter = Filter::equals(&constellation.to_string()).unwrap();
                                let focused_sp3 = sp3.filter(&filter);
                                let focused_nav = nav.filter(&filter);
                                reports.insert(
                                    constellation,
                                    BrdcSp3Report::new(&focused_sp3, &focused_nav),
                                );
                            }
                        }
                    }
                }
                reports
            },
        }
    }
    pub fn html_inline_menu_bar(&self) -> Markup {
        html! {
            a id="menu:orbit" {
                span class="icon" {
                    i class="fa-solid fa-globe" {}
                }
                "Orbital projections"
            }
        }
    }
}

#[cfg(feature = "sp3")]
impl Render for OrbitReport {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tr {
                        th class="is-info" {
                            "Map projection"
                        }
                        td {
                            (self.map_proj.render())
                        }
                    }
                    //tr {
                    //    th class="is-info" {
                    //        "Globe projection"
                    //    }
                    //    td {
                    //        (self.globe_proj.render())
                    //    }
                    //}
                    tr {
                        th class="is-info" {
                            "Sky plot"
                        }
                        td {
                            (self.sky_plot.render())
                        }
                    }
                    tr {
                        th class="is-info" {
                            "Elevation"
                        }
                        td {
                            (self.elev_plot.render())
                        }
                    }
                    @if self.brdc_sp3_err.len() > 0 {
                        @for (constell, page) in self.brdc_sp3_err.iter() {
                            tr {
                                th class="is-info" {
                                    (format!("{} SP3/BRDC", constell))
                                }
                                td {
                                    (page.render())
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(not(feature = "sp3"))]
impl Render for OrbitReport {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tr {
                        th class="is-info" {
                            "Map projection"
                        }
                        td {
                            (self.map_proj.render())
                        }
                    }
                    //tr {
                    //    th class="is-info" {
                    //        "Globe projection"
                    //    }
                    //    td {
                    //        (self.globe_proj.render())
                    //    }
                    //}
                    tr {
                        th class="is-info" {
                            "Sky plot"
                        }
                        td {
                            (self.sky_plot.render())
                        }
                    }
                    tr {
                        th class="is-info" {
                            "Elevation"
                        }
                        td {
                            (self.elev_plot.render())
                        }
                    }
                }
            }
        }
    }
}
