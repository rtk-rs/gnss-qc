use maud::{html, Markup, Render};
use rinex::prelude::{Duration, Epoch, Rinex};

#[cfg(feature = "sp3")]
use sp3::SP3;

/// [SamplingReport] applies to all time domain products
pub struct SamplingReport {
    /// Total epochs
    pub total: usize,
    /// First [`Epoch`] identified in time
    pub first_epoch: Epoch,
    /// Last [`Epoch`] identified in time
    pub last_epoch: Epoch,
    /// Time span of this RINEX context
    pub duration: Duration,
    /// File [`Header`] sample rate
    pub sampling_interval: Option<Duration>,
    /// Dominant sample rate
    pub dominant_sample_rate: Option<f64>,
    /// Unusual data gaps
    pub gaps: Vec<(Epoch, Duration)>,
    /// longest gap detected
    pub longest_gap: Option<(Epoch, Duration)>,
    /// shortest gap detected
    pub shortest_gap: Option<(Epoch, Duration)>,
}

impl SamplingReport {
    pub fn from_rinex(rinex: &Rinex) -> Self {
        let gaps = rinex.data_gaps(None).collect::<Vec<_>>();
        Self {
            total: rinex.epoch_iter().count(),
            first_epoch: rinex
                .first_epoch()
                .expect("failed to determine first RINEX epoch, badly formed?"),
            last_epoch: rinex
                .last_epoch()
                .expect("failed to determine last RINEX epoch, badly formed?"),
            duration: rinex
                .duration()
                .expect("failed to determine RINEX time frame, badly formed?"),
            sampling_interval: rinex.sampling_interval(),
            dominant_sample_rate: rinex.dominant_sampling_rate_hz(),
            shortest_gap: gaps
                .iter()
                .min_by(|(_, dur_a), (_, dur_b)| dur_a.partial_cmp(dur_b).unwrap())
                .copied(),
            longest_gap: gaps
                .iter()
                .max_by(|(_, dur_a), (_, dur_b)| dur_a.partial_cmp(dur_b).unwrap())
                .copied(),
            gaps,
        }
    }
    #[cfg(feature = "sp3")]
    pub fn from_sp3(sp3: &SP3) -> Self {
        let t_start = sp3
            .first_epoch()
            .expect("undetermined first epoch: SP3 format error");

        let t_end = sp3
            .last_epoch()
            .expect("undetermined last epoch: SP3 format error");
        Self {
            total: sp3.epochs_iter().count(),
            gaps: Vec::new(),   // TODO
            longest_gap: None,  //TODO
            shortest_gap: None, //TODO
            last_epoch: t_end,
            first_epoch: t_start,
            duration: t_end - t_start,
            sampling_interval: Some(sp3.header.sampling_period),
            dominant_sample_rate: Some(1.0 / sp3.header.sampling_period.to_seconds()),
        }
    }
}

impl Render for SamplingReport {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tbody {
                        tr {
                            th class="is-info" {
                                "Time Frame"
                            }
                        }
                        tr {
                            th {
                                "Start"
                            }
                            td {
                                (self.first_epoch.to_string())
                            }
                            th {
                                "End"
                            }
                            td {
                                (self.last_epoch.to_string())
                            }
                        }
                        tr {
                            th {
                                "Duration"
                            }
                            td {
                                (self.duration.to_string())
                            }
                        }
                        @if let Some(sampling_interval) = self.sampling_interval {
                            tr {
                                th class="is-info" {
                                    "Sampling Period"
                                }
                            }
                            tr {
                                td {
                                    (sampling_interval.to_string())
                                }
                            }
                        }
                        @if let Some(sample_rate) = self.dominant_sample_rate {
                            tr {
                                th class="is-info" {
                                    "Dominant Sampling Rate"
                                }
                            }
                            tr {
                                td {
                                    (format!("{:.3} Hz", sample_rate))
                                }
                            }
                        }
                        @if self.gaps.len() == 0 {
                            tr {
                                th class="is-primary" {
                                    "Data gaps"
                                }
                                td {
                                    "No gaps detected"
                                }
                            }
                        } @else {
                            tr {
                                th class="is-warning" {
                                    "Data gaps"
                                }
                                td {
                                    (format!("{} Data gaps", self.gaps.len()))
                                }
                            }
                            @if let Some((t_start, dur)) = self.shortest_gap {
                                tr {
                                    th {
                                        "Shortest"
                                    }
                                    td {
                                        (t_start.to_string())
                                    }
                                    td {
                                        (dur.to_string())
                                    }
                                }
                            }
                            @if let Some((t_start, dur)) = self.longest_gap {
                                tr {
                                    th {
                                        "Longest"
                                    }
                                    td {
                                        (t_start.to_string())
                                    }
                                    td {
                                        (dur.to_string())
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
