use crate::block::DspBlockConv;
use volk_rs::vec::AlignedVec;
use volk_rs::Complex;
use crate::dc_block::DcBlock;
use crate::block::DspBlock;

pub struct AmDemod {
    dc_block: DcBlock::<f32>,
}

impl AmDemod {
    pub fn new() -> Self {
        AmDemod {
            dc_block: DcBlock::<f32>::new(0.01),
        }
    }

    pub fn process(&mut self, input: &[Complex<f32>], output: &mut [f32]) {
        volk_rs::kernels::volk_32fc_magnitude_32f(output, input);
        self.dc_block.process_in_place(output);
        // TODO: probably needs an AGC at the output
    }
}

impl DspBlockConv<Complex<f32>, f32> for AmDemod {
    fn process(&mut self, input: &[Complex<f32>], output: &mut [f32]) {
        self.process(input, output);
    }

    fn compute_output_size(&mut self, input_size: usize) -> usize {
        input_size
    }

    fn set_input_size(&mut self, input_size: usize) {}
}
