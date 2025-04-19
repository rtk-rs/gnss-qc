use rinex::prelude::{nav::Orbit, Constellation, Epoch, Rinex, SV};
use std::collections::{BTreeMap, HashMap};

use crate::{
    plot::{MapboxStyle, MarkerSymbol, Mode},
    prelude::{html, Markup, Plot, QcContext, Render},
};

/// [OrbitalReport] without SP3 support.
pub struct OrbitalReport {
    sky_plot: Plot,
    elev_plot: Plot,
    map_proj: Plot,
}

impl OrbitalReport {
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
