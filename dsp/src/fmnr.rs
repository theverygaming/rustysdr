use volk_rs::vec::AlignedVec;
use fftw::plan::*;
use fftw::types::*;
use volk_rs::Complex;
use crate::block::DspBlock;


pub struct FMNr {
    fft_size: usize,
    block_size: usize,
    magnitude_buf: AlignedVec<f32>,
    delay_buf: AlignedVec<Complex<f32>>,
    fft_in_fwd: AlignedVec<Complex<f32>>,
    fft_out_fwd: AlignedVec<Complex<f32>>,
    fft_in_bwd: AlignedVec<Complex<f32>>,
    fft_out_bwd: AlignedVec<Complex<f32>>,
    fft_plan_fwd: C2CPlan32,
    fft_plan_bwd: C2CPlan32,
}

impl FMNr {
    pub fn new(fft_size: usize, block_size: usize) -> Self {
        assert!(fft_size <= block_size, "FFT size larger than block size");
        assert!((fft_size > 2) && ((fft_size & (fft_size -1)) == 0), "FFT size must be a power of two");
        FMNr {
            fft_size: fft_size,
            block_size: block_size,
            magnitude_buf: AlignedVec::new_zeroed(fft_size),
            delay_buf: AlignedVec::new_zeroed(block_size + fft_size),
            fft_in_fwd: AlignedVec::new_zeroed(fft_size),
            fft_out_fwd: AlignedVec::new_zeroed(fft_size),
            fft_in_bwd: AlignedVec::new_zeroed(fft_size),
            fft_out_bwd: AlignedVec::new_zeroed(fft_size),
            fft_plan_fwd: C2CPlan32::aligned(&[fft_size], Sign::Forward, Flag::ESTIMATE).unwrap(),
            fft_plan_bwd: C2CPlan32::aligned(&[fft_size], Sign::Backward, Flag::ESTIMATE).unwrap(),
        }
    }
}

impl DspBlock<Complex<f32>> for FMNr {
    fn process(&mut self, input: &mut [Complex<f32>], output: &mut [Complex<f32>]) {
        self.delay_buf[self.fft_size..self.fft_size+self.block_size].copy_from_slice(input);
        for i in 0..output.len()-1 {
            // TODO: window
            self.fft_in_fwd.copy_from_slice(&self.delay_buf[i..i+self.fft_size]); // TODO: remove when window
            
            self.fft_plan_fwd.c2c(&mut self.fft_in_fwd, &mut self.fft_out_fwd).unwrap();

            let mut peak_idx = 0;
            volk_rs::kernels::volk_32fc_magnitude_32f(&mut self.magnitude_buf, &self.fft_out_fwd);
            volk_rs::kernels::volk_32f_index_max_32u(&self.magnitude_buf, &mut peak_idx);

            self.fft_in_bwd[peak_idx as usize] = self.fft_out_fwd[peak_idx as usize];

            self.fft_plan_bwd.c2c(&mut self.fft_in_bwd, &mut self.fft_out_bwd).unwrap();
            self.fft_in_bwd[peak_idx as usize] = Complex {re: 0.0, im: 0.0};

            output[i] = self.fft_out_bwd[0];
        }
        self.delay_buf.copy_within(self.block_size..self.fft_size+self.block_size, 0);
    }
}
