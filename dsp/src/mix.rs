use volk_rs::Complex;
use crate::block::DspBlock;

// maybe call it frequencyXlator instead? https://github.com/randomradioprojects/SDRPlusPlus/blob/0487aa9e28e8b3ca7d82aef305301fb57b154329/core/src/dsp/channel/frequency_xlator.h#L44
pub struct Mixer {
    phase_inc: Complex<f32>,
    phase: Complex<f32>,
}

impl Mixer {
    pub fn new(lofreq: f64, samplerate: f64) -> Self {
        let phase: f64 = (2.0 * std::f64::consts::PI) * (lofreq / samplerate);
        Mixer {
            phase_inc: Complex {
                re: phase.cos() as f32,
                im: phase.sin() as f32,
            },
            phase: Complex { re: 1.0, im: 0.0 },
        }
    }

    pub fn set(&mut self, lofreq: f64, samplerate: f64) {
        let phase: f64 = (2.0 * std::f64::consts::PI) * (lofreq / samplerate);
        self.phase_inc = Complex {
            re: phase.cos() as f32,
            im: phase.sin() as f32,
        };
    }
}

impl DspBlock<Complex<f32>> for Mixer {
    fn process(&mut self, input: &mut [Complex<f32>], output: &mut [Complex<f32>]) {
        volk_rs::kernels::volk_32fc_s32fc_x2_rotator_32fc(input, output, self.phase_inc, &mut self.phase);
    }
}
