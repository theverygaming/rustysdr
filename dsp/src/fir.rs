use volk_rs::vec::AlignedVec;
use volk_rs::Complex;
use crate::block::DspBlock;


pub struct FirFilter {
    block_size: usize,
    delay_buf: AlignedVec<Complex<f32>>,
    taps: AlignedVec<f32>,
}

impl FirFilter {
    pub fn new() -> Self {
        FirFilter {
            block_size: 1,
            delay_buf: AlignedVec::new_zeroed(2),
            taps: AlignedVec::new_zeroed(1),
        }
    }

    pub fn set_taps(&mut self, taps: AlignedVec<f32>) {
        let old_len = self.taps.len();
        self.taps = taps;
        self.delay_buf = AlignedVec::new_zeroed((self.delay_buf.len() - old_len) + self.taps.len());
    }
}

impl DspBlock<Complex<f32>> for FirFilter {
    fn process(&mut self, input: &mut [Complex<f32>], output: &mut [Complex<f32>]) {
        let n_taps = self.taps.len();
        self.delay_buf[n_taps..n_taps+self.block_size].copy_from_slice(input);

        for i in 0..output.len() {
            volk_rs::kernels::volk_32fc_32f_dot_prod_32fc(&self.delay_buf[i..i+n_taps], &mut output[i], &self.taps);
        }

        self.delay_buf.copy_within(self.block_size..n_taps+self.block_size, 0);
    }

    fn compute_output_size(&mut self, input_size: usize) -> usize {
        input_size
    }

    fn set_input_size(&mut self, input_size: usize) {
        assert!(input_size > 1, "input size must be larger than one");
        self.block_size = input_size;
        self.delay_buf = AlignedVec::new_zeroed(self.block_size + self.taps.len());
    }
}
