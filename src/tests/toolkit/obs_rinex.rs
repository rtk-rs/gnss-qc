use rinex::prelude::{Header, Rinex};

pub fn header_comparison_eq(model: &Header, dut: &Header) {
    assert_eq!(model.version, dut.version);
    assert_eq!(model.agency, dut.agency);
    assert_eq!(model.date, dut.date);
    assert_eq!(model.cospar, dut.cospar);

    let model_hd = model
        .obs
        .as_ref()
        .expect("proposed model is not valid OBSERVATION RINEX!");
    let dut_hd = dut
        .obs
        .as_ref()
        .expect("proposed DUT is not valid OBSERVATION RINEX!");

    assert_eq!(model_hd.clock_offset_applied, dut_hd.clock_offset_applied);
    assert_eq!(model_hd.codes, dut_hd.codes);
    assert_eq!(model_hd.crinex, dut_hd.crinex);

    if let Some(timeof_first_obs) = model_hd.timeof_first_obs {
        let timeof_first_obs_dut = dut_hd
            .timeof_first_obs
            .expect("missing DUT TIME OF FIRST OBS");
        assert_eq!(
            timeof_first_obs, timeof_first_obs_dut,
            "TIME OF FIRST OBS mismatch"
        );
        assert_eq!(
            timeof_first_obs.time_scale, timeof_first_obs_dut.time_scale,
            "TIME OF FIRST OBS: timescale mismatch"
        );
    }

    if let Some(timeof_last_obs) = model_hd.timeof_last_obs {
        let timeof_last_obs_dut = dut_hd
            .timeof_last_obs
            .expect("missing DUT TIME OF LAST OBS");
        assert_eq!(
            timeof_last_obs, timeof_last_obs_dut,
            "TIME OF LAST OBS mismatch"
        );
        assert_eq!(
            timeof_last_obs.time_scale, timeof_last_obs_dut.time_scale,
            "TIME OF LAST OBS: timescale mismatch"
        );
    }
}

pub fn rinex_comparison_eq(model: &Rinex, dut: &Rinex) {
    assert!(
        model.is_observation_rinex(),
        "proposed model is not OBSERVATION RINEX!"
    );
    assert!(
        dut.is_observation_rinex(),
        "proposed DUT is not OBSERVATION RINEX!"
    );

    header_comparison_eq(&model.header, &dut.header);

    let model_rec = model.record.as_obs().unwrap();
    let dut_rec = dut.record.as_obs().unwrap();

    for (k_model, v_model) in model_rec.iter() {
        let v_dut = dut_rec
            .get(k_model)
            .unwrap_or_else(|| panic!("DUT missing entry at {:?}", k_model));

        assert_eq!(
            v_model.clock, v_dut.clock,
            "Clock data mismatch at {:?}",
            k_model
        );

        for (index, model_sig) in v_model.signals.iter().enumerate() {
            let dut_sig = v_dut
                .signals
                .get(index)
                .unwrap_or_else(|| panic!("DUT missing signal {:?} at {:?}", model_sig, k_model));

            assert_eq!(model_sig.lli, dut_sig.lli);
            assert_eq!(model_sig.snr, dut_sig.snr);
            assert!((model_sig.value - dut_sig.value).abs() < 1E-5);
        }
    }

    for (k_dut, v_dut) in dut_rec.iter() {
        let v_model = model_rec
            .get(k_dut)
            .unwrap_or_else(|| panic!("DUT has unexpected entry at {:?}", k_dut));

        assert_eq!(
            v_model.clock, v_dut.clock,
            "Clock data mismatch at {:?}",
            k_dut
        );

        for (index, dut_sig) in v_dut.signals.iter().enumerate() {
            let model_sig = v_model.signals.get(index).unwrap_or_else(|| {
                panic!("DUT has unexpected signal {:?} at {:?}", dut_sig, k_dut)
            });

            assert_eq!(model_sig.lli, dut_sig.lli);
            assert_eq!(model_sig.snr, dut_sig.snr);
            assert!((model_sig.value - dut_sig.value).abs() < 1E-5);
        }
    }
}
