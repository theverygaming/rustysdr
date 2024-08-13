use crate::block::DspBlock;
use crate::filters;
use volk_rs::vec::AlignedVec;
use volk_rs::Complex;

pub struct RationalResampler {
    decimation: u32,
    interpolation: u32,
    block_size: usize,
    delay_buf: AlignedVec<Complex<f32>>,
    filters: std::vec::Vec<AlignedVec<f32>>,
    offset: u32,
    phase: u32,
}

fn generate_polyphase_bank(nphases: usize, taps: &[f32]) -> std::vec::Vec<AlignedVec<f32>> {
    // https://github.com/AlexandreRouma/SDRPlusPlus/blob/67520ea45e57b17e815655c71713779a638d648a/core/src/dsp/multirate/polyphase_bank.h#L15
    let taps_per_phase = (taps.len() + nphases - 1) / nphases;
    let mut phases = vec![AlignedVec::new_zeroed(taps_per_phase); nphases];

    let total_taps = nphases * taps_per_phase;
    for i in 0..total_taps {
        phases[(nphases - 1) - (i % nphases)][i / nphases] = if i < taps.len() { taps[i] } else { 0.0 };
    }
    phases
}

fn design_resamp_filter(interpolation: f32, decimation: f32, fractional_bw: f32) -> AlignedVec<f32> {
    // https://github.com/SatDump/SatDump/blob/533f91b546a1d0909a7395550c531d2ddce8b6c0/src-core/common/dsp/filter/firdes.cpp#L127
    let halfband = 0.5;
    let rate = interpolation / decimation;
    let trans_width;
    let mid_transition_band;

    if rate >= 1.0 {
        trans_width = halfband - fractional_bw;
        mid_transition_band = halfband - trans_width / 2.0;
    } else {
        trans_width = rate * (halfband - fractional_bw);
        mid_transition_band = rate * halfband - trans_width / 2.0;
    }

    let mut n_taps = (92.0 * interpolation / (22.0 * trans_width)) as usize; // 92 -> max attenuation of blackman-harris window
    if (n_taps % 2) == 0 {
        n_taps += 1;
    }

    let mut taps = AlignedVec::new_zeroed(n_taps);
    filters::lowpass(&mut taps, interpolation, mid_transition_band, interpolation);
    taps
}

impl RationalResampler {
    pub fn new(interpolation: u32, decimation: u32) -> Self {
        assert!((interpolation > 0) && (decimation > 0), "interpolation and decimation may not be zero");
        let taps = design_resamp_filter(interpolation as f32, decimation as f32, 0.4);
        let polyphase_bank = generate_polyphase_bank(interpolation as usize, &taps);
        RationalResampler {
            decimation: decimation,
            interpolation: interpolation,
            block_size: polyphase_bank[0].len(),
            delay_buf: AlignedVec::new_zeroed(polyphase_bank[0].len() + polyphase_bank[0].len()),
            filters: polyphase_bank,
            offset: 0,
            phase: 0,
        }
    }
}

impl DspBlock<Complex<f32>> for RationalResampler {
    fn process(&mut self, input: &mut [Complex<f32>], output: &mut [Complex<f32>]) {
        assert!(input.len() == self.block_size);
        assert!(output.len() == self.compute_output_size(input.len()), "incorrect output size");
        let mut out_idx = 0;
        let n_taps = self.filters[0].len();
        self.delay_buf[n_taps..n_taps + self.block_size].copy_from_slice(input);

        while self.offset < input.len() as u32 {
            // https://github.com/AlexandreRouma/SDRPlusPlus/blob/67520ea45e57b17e815655c71713779a638d648a/core/src/dsp/multirate/polyphase_resampler.h#L75
            volk_rs::kernels::volk_32fc_32f_dot_prod_32fc(
                &self.delay_buf[(self.offset as usize)..(self.offset as usize) + n_taps],
                &mut output[out_idx],
                &self.filters[self.phase as usize],
            );
            out_idx += 1;

            self.phase += self.decimation;
            self.offset += self.phase / self.interpolation;
            self.phase %= self.interpolation;
        }

        self.offset -= input.len() as u32;

        self.delay_buf.copy_within(self.block_size..n_taps + self.block_size, 0);
    }

    fn compute_output_size(&mut self, input_size: usize) -> usize {
        let rate_mult = (self.interpolation as f32) / (self.decimation as f32);
        ((input_size as f32) * rate_mult).ceil() as usize
    }

    fn set_input_size(&mut self, input_size: usize) {
        assert!(input_size > 1, "input size must be larger than one");
        self.block_size = input_size;
        self.delay_buf = AlignedVec::new_zeroed(self.block_size + self.filters[0].len());
    }
}
