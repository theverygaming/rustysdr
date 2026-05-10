use crate::block::DspBlock;
use crate::filters;
use volk_rs::vec::AlignedVec;
use volk_rs::Complex;

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

pub struct RationalResampler<Tsamples> {
    decimation: usize,
    interpolation: usize,
    delay_buf: AlignedVec<Tsamples>,
    taps: std::vec::Vec<AlignedVec<f32>>,
    phase: usize,
}

pub trait RationalResamplerDotProd<Tsamples> {
    fn dot_prod(&self, samps: &[Tsamples]) -> Tsamples;
}

impl RationalResamplerDotProd<f32> for RationalResampler<f32> {
    fn dot_prod(&self, samps: &[f32]) -> f32 {
        let mut x: f32 = 0.0;
        volk_rs::kernels::volk_32f_x2_dot_prod_32f(&mut x, samps, &self.taps[self.phase as usize]);
        x
    }
}

impl RationalResamplerDotProd<Complex<f32>> for RationalResampler<Complex<f32>> {
    fn dot_prod(&self, samps: &[Complex<f32>]) -> Complex<f32> {
        let mut x: Complex<f32> = Complex { re: 0.0, im: 0.0 };
        volk_rs::kernels::volk_32fc_32f_dot_prod_32fc(&mut x, samps, &self.taps[self.phase as usize]);
        x
    }
}

impl<Tsamples: Copy> RationalResampler<Tsamples>
where
    Self: RationalResamplerDotProd<Tsamples>,
{
    pub fn new(interpolation: usize, decimation: usize) -> Self {
        assert!((interpolation > 0) && (decimation > 0), "interpolation and decimation may not be zero");
        let taps = design_resamp_filter(interpolation as f32, decimation as f32, 0.4);
        let polyphase_bank = generate_polyphase_bank(interpolation as usize, &taps);
        RationalResampler {
            decimation: decimation,
            interpolation: interpolation,
            delay_buf: AlignedVec::new_zeroed(polyphase_bank[0].len() * 2),
            taps: polyphase_bank,
            phase: 0,
        }
    }

    pub fn set_block_size(&mut self, block_size: usize) {
        self.delay_buf = AlignedVec::new_zeroed(self.taps[0].len() + self.taps[0].len() + block_size);
    }

    // FIXME: this might be kinda broken
    pub fn process(&mut self, input: &[Tsamples], output: &mut [Tsamples]) -> usize {
        let n_taps = self.taps[0].len();
        let block_size = self.delay_buf.len() - n_taps;

        let mut in_idx = 0;
        let mut out_idx = 0;

        while in_idx < input.len() {
            let n = std::cmp::min(input.len() - in_idx, block_size);

            // copy old samples to the start of the buffer
            self.delay_buf.copy_within(n..n + n_taps - 1, 0);

            // add new input samples to the buffer
            self.delay_buf[n_taps - 1..n_taps - 1 + n].copy_from_slice(&input[in_idx..in_idx + n]);
            
            let mut offset = 0;
            while offset <= n - 1 {
                output[out_idx] = self.dot_prod(&self.delay_buf[offset..offset + n_taps]);

                self.phase += self.decimation;
                offset += self.phase / self.interpolation;
                self.phase %= self.interpolation;

                out_idx += 1;
            }

            in_idx += offset;
        }

        return out_idx;
    }

    fn get_max_out_size(&self, ninput: usize) -> usize {
        let rate_mult = (self.interpolation as f32) / (self.decimation as f32);
        ((ninput as f32) * rate_mult).ceil() as usize
    }
}

impl DspBlock<Complex<f32>> for RationalResampler<Complex<f32>> {
    fn process(&mut self, input: &[Complex<f32>], output: &mut [Complex<f32>]) {
        self.process(input, output);
    }

    fn compute_output_size(&mut self, input_size: usize) -> usize {
        self.get_max_out_size(input_size)
    }

    fn set_input_size(&mut self, input_size: usize) {
        assert!(input_size > 1, "input size must be larger than one");
        self.set_block_size(input_size);
    }
}
