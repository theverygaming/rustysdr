use std::option::Option;
use volk_rs::{types::complex, vec::AlignedVec};

// maybe call it frequencyXlator instead? https://github.com/randomradioprojects/SDRPlusPlus/blob/0487aa9e28e8b3ca7d82aef305301fb57b154329/core/src/dsp/channel/frequency_xlator.h#L44
pub struct Mixer {
    phase_inc: complex<f32>,
    phase: complex<f32>,
}

impl Mixer {
    pub fn new(lofreq: f64, samplerate: f64) -> Self {
        let phase: f64 = (2.0 * std::f64::consts::PI) * (lofreq / samplerate);
        Mixer {
            phase_inc: complex {
                r: phase.cos() as f32,
                i: phase.sin() as f32,
            },
            phase: complex { r: 1.0, i: 0.0 },
        }
    }

    pub fn set(&mut self, lofreq: f64, samplerate: f64) {
        let phase: f64 = (2.0 * std::f64::consts::PI) * (lofreq / samplerate);
        self.phase_inc = complex {
            r: phase.cos() as f32,
            i: phase.sin() as f32,
        };
    }

    pub fn run(&mut self, input: &mut AlignedVec<complex<f32>>, output_opt: Option<&mut AlignedVec<complex<f32>>>) {
        volk_rs::kernels::v32fc_s32fc_x2_rotator_32fc(output_opt, input, self.phase_inc, &mut self.phase);
    }
}
