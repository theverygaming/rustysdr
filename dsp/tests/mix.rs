#[test]
fn mixer_works() {
    let mut _mx = dsp::mix::Mixer::new(100.0, 5000.0);
    _mx.set(100.0, 5000.0);
}
