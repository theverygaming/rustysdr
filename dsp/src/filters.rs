use crate::windows;

pub fn lowpass(taps: &mut [f32], samplerate: f32, cutoff: f32, mut gain: f32) {
    assert!((taps.len() % 2) == 1, "taps must be odd");
    let mut window = vec![0.0; taps.len()];
    windows::blackmanharris(&mut window);

    // https://github.com/theverygaming/shitDSP/blob/b89cba32fad5f8fece34b7e13cde50cc98356b9e/dsp/firfilters.h#L32
    let sampletime = 1.0 / samplerate;
    for i in 0..taps.len() {
        let x = (i as isize) - (taps.len() / 2) as isize;
        if x == 0 {
            taps[i] = 2.0 * cutoff;
            continue;
        }
        let xf = x as f32;
        taps[i] = (f32::sin(2.0 * std::f32::consts::PI * cutoff * sampletime * xf) / (std::f32::consts::PI * sampletime * xf)) * window[i];
    }

    // normalize
    // https://github.com/SatDump/SatDump/blob/ecb05a11dee9a9c927d9d3c981c28dcdc1b8e74b/src-core/common/dsp/filter/firdes.cpp#L113
    let mut fmax = taps[taps.len() / 2];
    for i in (taps.len() / 2)+1..taps.len() {
        fmax += 2.0 * taps[i];
    }
    gain /= fmax;
    for i in 0..taps.len() {
        taps[i] *= gain;
    }
}
