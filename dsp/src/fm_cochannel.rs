use crate::block::DspBlock;
use crate::windows;
use fftw::plan::*;
use fftw::types::*;
use volk_rs::vec::AlignedVec;
use volk_rs::Complex;

pub struct FMCochannelCancel {
    fft_size: usize,
    block_size: usize,
    n_neighbours: usize,
    magnitude_buf: AlignedVec<f32>,
    delay_buf: AlignedVec<Complex<f32>>,
    fft_window: AlignedVec<f32>,
    fft_in_fwd: AlignedVec<Complex<f32>>,
    fft_out_fwd: AlignedVec<Complex<f32>>,
    fft_out_bwd: AlignedVec<Complex<f32>>,
    fft_plan_fwd: C2CPlan32,
    fft_plan_bwd: C2CPlan32,
}

impl FMCochannelCancel {
    pub fn new(fft_size: usize, n_neighbours: usize) -> Self {
        assert!((n_neighbours % 2) == 0, "number of neighbours must be even");
        assert!((fft_size > 2) && ((fft_size & (fft_size - 1)) == 0), "FFT size must be a power of two");
        let mut nr = FMCochannelCancel {
            fft_size: fft_size,
            block_size: fft_size,
            n_neighbours: n_neighbours * 2,
            magnitude_buf: AlignedVec::new_zeroed(fft_size),
            delay_buf: AlignedVec::new_zeroed(fft_size + fft_size),
            fft_window: AlignedVec::new_zeroed(fft_size),
            fft_in_fwd: AlignedVec::new_zeroed(fft_size),
            fft_out_fwd: AlignedVec::new_zeroed(fft_size),
            fft_out_bwd: AlignedVec::new_zeroed(fft_size),
            fft_plan_fwd: C2CPlan32::aligned(&[fft_size], Sign::Forward, Flag::ESTIMATE).unwrap(),
            fft_plan_bwd: C2CPlan32::aligned(&[fft_size], Sign::Backward, Flag::ESTIMATE).unwrap(),
        };
        windows::nuttall(&mut nr.fft_window);
        nr
    }
}

impl DspBlock<Complex<f32>> for FMCochannelCancel {
    fn process(&mut self, input: &mut [Complex<f32>], output: &mut [Complex<f32>]) {
        assert!(input.len() == self.block_size);
        self.delay_buf[self.fft_size..self.fft_size + self.block_size].copy_from_slice(input);
        for i in 0..output.len() {
            volk_rs::kernels::volk_32fc_32f_multiply_32fc(&self.delay_buf[i..i + self.fft_size], &mut self.fft_in_fwd, &self.fft_window);

            self.fft_plan_fwd.c2c(&mut self.fft_in_fwd, &mut self.fft_out_fwd).unwrap();

            let mut _peak_idx = 0;
            volk_rs::kernels::volk_32fc_magnitude_32f(&mut self.magnitude_buf, &self.fft_out_fwd);
            volk_rs::kernels::volk_32f_index_max_32u(&self.magnitude_buf, &mut _peak_idx);
            let peak_idx = _peak_idx as usize;

            self.fft_out_fwd[peak_idx] = Complex { re: 0.0, im: 0.0 };
            for i in 1..=self.n_neighbours {
                if peak_idx >= i {
                    self.fft_out_fwd[peak_idx - i] = Complex { re: 0.0, im: 0.0 };
                }
                if peak_idx + i < self.fft_out_fwd.len() {
                    self.fft_out_fwd[peak_idx + i] = Complex { re: 0.0, im: 0.0 };
                }
            }

            self.fft_plan_bwd.c2c(&mut self.fft_out_fwd, &mut self.fft_out_bwd).unwrap();

            output[i] = self.fft_out_bwd[self.fft_out_bwd.len() / 2];
        }
        self.delay_buf.copy_within(self.block_size..self.fft_size + self.block_size, 0);
    }

    fn compute_output_size(&mut self, input_size: usize) -> usize {
        input_size
    }

    fn set_input_size(&mut self, input_size: usize) {
        assert!(self.fft_size <= input_size, "FFT size larger than block size");
        self.block_size = input_size;
        self.delay_buf = AlignedVec::new_zeroed(self.block_size + self.fft_size);
    }
}
