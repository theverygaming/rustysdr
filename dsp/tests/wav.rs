use std::fs::File;
use volk_rs::{self, vec::AlignedVec};

#[test]
fn wav_works() {
    let mut testsamps: AlignedVec<f32> = AlignedVec::from_elem(0.5, 148000);
    let mut readbuf: AlignedVec<f32> = AlignedVec::from_elem(0.5, 148000);
    let mut c: f32 = -1.0;
    let mut i: f32 = 0.003;
    for e in testsamps.iter_mut() {
        if c >= 0.999 {
            c = 0.999;
            i = -i;
        }
        if c <= -0.999 {
            c = -0.999;
            i = i.abs();
            i += 0.005;
            if i >= 1.0 {
                i = 0.003;
            }
        }
        *e = c;
        c += i;
    }

    fn test_samp_format(mut readbuf: &mut [f32], testsamps: &AlignedVec<f32>, format: dsp::wav::WavSampleFormat) {
        let sr = 48000;
        let channels = 1;
        let mut wav_w = dsp::wav::Writer::new(File::create(std::env::temp_dir().join("wav-works.wav")).unwrap(), sr, channels, format).expect("couldn't open wav");
        wav_w.write_samples(&testsamps).unwrap();
        wav_w.flush().unwrap();
        let mut wav_r = dsp::wav::Reader::new(File::open(std::env::temp_dir().join("wav-works.wav")).unwrap(), true).expect("couldn't open wav");

        assert!(format == wav_r.get_sample_format(), "WAV format mismatch");
        assert!(sr == wav_r.get_samplerate(), "WAV samplerate mismatch");
        assert!(channels == wav_r.get_channels(), "WAV channel count mismatch");
        assert!(testsamps.len() as u64 == wav_r.get_sample_count().unwrap(), "WAV sample count mismatch");

        wav_r.read_samples(&mut readbuf).unwrap();
        let mut n = 0;
        for e in testsamps.iter().zip(readbuf.iter()) {
            let (test, read) = e;
            let acc = (test - read).abs();
            if acc > 0.01 {
                panic!("value off by: {} (format: {:?} index: {}) got: {} expected: {}", acc, format, n, read, test);
            }
            n += 1;
        }
    }

    test_samp_format(&mut readbuf, &testsamps, dsp::wav::WavSampleFormat::U8);
    test_samp_format(&mut readbuf, &testsamps, dsp::wav::WavSampleFormat::S16);
    test_samp_format(&mut readbuf, &testsamps, dsp::wav::WavSampleFormat::S32);
    test_samp_format(&mut readbuf, &testsamps, dsp::wav::WavSampleFormat::F32);
    test_samp_format(&mut readbuf, &testsamps, dsp::wav::WavSampleFormat::F64);
}
