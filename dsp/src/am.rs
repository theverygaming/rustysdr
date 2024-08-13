use crate::block::DspBlockConv;
use volk_rs::vec::AlignedVec;
use volk_rs::Complex;
//use crate::dc_block::DcBlock;
use crate::block::DspBlock;

pub struct AmDemod {
    //dc_block: DcBlock<f32>,
}

impl AmDemod {
    pub fn new() -> Self {
        AmDemod {
            //dc_block: DcBlock::<f32>::new()
        }
    }
}

impl DspBlockConv<Complex<f32>, f32> for AmDemod {
    fn process(&mut self, input: &mut [Complex<f32>], output: &mut [f32]) {
        let mut tmp = AlignedVec::new_zeroed(output.len());
        volk_rs::kernels::volk_32fc_magnitude_32f(&mut tmp, input);
        //self.dc_block.process(&mut tmp, output);
    }
}
