use crate::block::DspBlock;
use volk_rs::vec::AlignedVec;
use volk_rs::Complex;

pub struct FirFilter<Tsamples, Ttaps> {
    delay_buf: AlignedVec<Tsamples>,
    taps: AlignedVec<Ttaps>,
}

pub trait FirFilterDotProd<Tsamples, Ttaps> {
    fn dot_prod(&self, samps: &[Tsamples]) -> Tsamples;
}

impl FirFilterDotProd<f32, f32> for FirFilter<f32, f32> {
    fn dot_prod(&self, samps: &[f32]) -> f32 {
        let mut x: f32 = 0.0;
        volk_rs::kernels::volk_32f_x2_dot_prod_32f(&mut x, samps, &self.taps);
        x
    }
}

impl FirFilterDotProd<Complex<f32>, f32> for FirFilter<Complex<f32>, f32> {
    fn dot_prod(&self, samps: &[Complex<f32>]) -> Complex<f32> {
        let mut x: Complex<f32> = Complex { re: 0.0, im: 0.0 };
        volk_rs::kernels::volk_32fc_32f_dot_prod_32fc(&mut x, samps, &self.taps);
        x
    }
}

impl FirFilterDotProd<Complex<f32>, Complex<f32>> for FirFilter<Complex<f32>, Complex<f32>> {
    fn dot_prod(&self, samps: &[Complex<f32>]) -> Complex<f32> {
        let mut x: Complex<f32> = Complex { re: 0.0, im: 0.0 };
        volk_rs::kernels::volk_32fc_x2_dot_prod_32fc(&mut x, samps, &self.taps);
        x
    }
}

impl<Tsamples: Copy, Ttaps: Copy> FirFilter<Tsamples, Ttaps>
where
    Self: FirFilterDotProd<Tsamples, Ttaps>,
{
    pub fn new(taps: AlignedVec<Ttaps>, block_size: usize) -> Self {
        FirFilter {
            delay_buf: AlignedVec::new_zeroed(taps.len() + block_size),
            taps: taps,
        }
    }

    pub fn set_taps(&mut self, taps: AlignedVec<Ttaps>) {
        let old_taps_len = self.taps.len();
        self.taps = taps;
        self.delay_buf = AlignedVec::new_zeroed(self.taps.len() + (self.delay_buf.len() - old_taps_len));
    }

    pub fn set_block_size(&mut self, block_size: usize) {
        self.delay_buf = AlignedVec::new_zeroed(self.taps.len() + block_size);
    }

    // FIXME: this might be kinda broken, we fixed the resampler block, maybe we can fix this one too?
    pub fn process(&mut self, input: &[Tsamples], output: &mut [Tsamples]) {
        assert!(input.len() == output.len(), "mismatched lengths");

        let n_taps = self.taps.len();
        let block_size = self.delay_buf.len() - n_taps;

        let mut data_idx = 0;

        while data_idx < input.len() {
            let n = std::cmp::min(input.len() - data_idx, block_size);

            // copy old samples to the start of the buffer
            self.delay_buf.copy_within(n..n + n_taps - 1, 0);

            // add new input samples to the buffer
            self.delay_buf[n_taps - 1..n_taps - 1 + n].copy_from_slice(&input[data_idx..data_idx + n]);

            for i in 0..n {
                output[data_idx + i] = self.dot_prod(&self.delay_buf[i..i + n_taps]);
            }

            data_idx += n;
        }
    }
}

#[test]
fn fir_works() {
    let mut fir = FirFilter::<Complex<f32>, f32>::new(AlignedVec::new_zeroed(31), 10);
    let mut input: AlignedVec<Complex<f32>> = AlignedVec::new_zeroed(500);
    let mut output: AlignedVec<Complex<f32>> = AlignedVec::new_zeroed(500);

    fir.process(&mut input[..1], &mut output[..1]);
    fir.process(&mut input[..10], &mut output[..10]);
    fir.process(&mut input[..31], &mut output[..31]);
    fir.process(&mut input[..31 + 10], &mut output[..31 + 10]);
    fir.process(&mut input[..100], &mut output[..100]);
    fir.process(&mut input[..500], &mut output[..500]);
}


impl DspBlock<Complex<f32>> for FirFilter<Complex<f32>, f32> {
    fn process(&mut self, input: &[Complex<f32>], output: &mut [Complex<f32>]) {
        self.process(input, output);
    }

    fn compute_output_size(&mut self, input_size: usize) -> usize {
        input_size
    }

    fn set_input_size(&mut self, input_size: usize) {
        self.set_block_size(input_size);
    }
}
