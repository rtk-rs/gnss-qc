use crate::{
    prelude::{QcContext, TimeScale},
    tests::toolkit::obs_rinex::rinex_comparison_eq as obs_rinex_comparison_eq,
};

use qc_traits::TimePolynomial;

use rinex::{observation::Record as ObsRecord, prelude::Header};

fn verify_dut_header(header: &Header, timescale: TimeScale) {
    let obs = header.obs.as_ref().expect("invalid DUT output header");

    let timeof_first = obs.timeof_first_obs.unwrap();
    let timeof_last = obs.timeof_last_obs.unwrap();

    assert_eq!(timeof_first.time_scale, timescale);
    assert_eq!(timeof_last.time_scale, timescale);
}

fn verify_transposed_dut(
    record: &ObsRecord,
    original: &ObsRecord,
    polynomial: TimePolynomial,
    timescale: TimeScale,
) {
}

#[test]
fn test_gps_gpst_timescale_transposition() {
    // verify that this operation does not change anything
    let mut context = QcContext::new();

    context
        .load_gzip_rinex_file("data/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz")
        .unwrap();

    context
        .load_gzip_rinex_file("data/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz")
        .unwrap();

    let original_obs = context.observation().unwrap();
    let _ = context.brdc_navigation().unwrap();

    let transposed = context.timescale_transposition(TimeScale::GPST);

    let transposed_obs = transposed.observation().unwrap();
    let _ = transposed.brdc_navigation().unwrap();

    // verify input is GPST
    assert_eq!(context.timescale(), Some(TimeScale::GPST));

    // verify transposed is GPST
    assert_eq!(transposed.timescale(), Some(TimeScale::GPST));

    obs_rinex_comparison_eq(&original_obs, &transposed_obs);
}

#[test]
fn test_gps_gst_timescale_transposition() {
    // verify that this operation does not change anything
    let mut context = QcContext::new();

    context
        .load_gzip_rinex_file("data/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz")
        .unwrap();

    context
        .load_gzip_rinex_file("data/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz")
        .unwrap();

    let original_obs = context.observation().unwrap();
    let _ = context.brdc_navigation().unwrap();

    let transposed = context.timescale_transposition(TimeScale::GST);

    let transposed_obs = transposed.observation().unwrap();
    let _ = transposed.brdc_navigation().unwrap();

    // verify input is GPST
    assert_eq!(context.timescale(), Some(TimeScale::GPST));

    // verify transposed is GST
    assert_eq!(transposed.timescale(), Some(TimeScale::GST));

    verify_dut_header(&transposed_obs.header, TimeScale::GST);
}
