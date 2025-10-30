use dsp::block::DspBlockConv;
use dsp::volk_rs::vec::AlignedVec;
use dsp::volk_rs::Complex;
use dsp::resamp::RationalResampler;
use dsp::chain::DspChain;
use dsp::mix::Mixer;
use dsp::block::DspBlock;
use std::{boxed::Box};

pub struct RealVFO {
    pub demod: Box<dyn DspBlockConv<Complex<f32>, f32>>,
    chain: DspChain<Complex<f32>>,
    buffer: AlignedVec<Complex<f32>>,
}

impl RealVFO {
    pub fn new(demod: Box<dyn DspBlockConv<Complex<f32>, f32>>) -> Self {
        let mut chain = DspChain::new();
        chain.add_block(Box::new(Mixer::new(-305000.0, 1024000 as f64)));
        chain.add_block(Box::new(RationalResampler::new(1, 40)));
        RealVFO {
            demod: demod,
            chain: chain,
            buffer: AlignedVec::new_zeroed(1048576),
        }
    }

    pub fn set_demod(&mut self, demod: Box<dyn DspBlockConv<Complex<f32>, f32>>) {
        self.demod = demod;
    }
}

impl DspBlockConv<Complex<f32>, f32> for RealVFO {
    fn process(&mut self, input: &[Complex<f32>], output: &mut [f32]) {
        self.chain.process(input, &mut self.buffer);
        self.demod.process(&mut self.buffer, output);
    }

    fn compute_output_size(&mut self, input_size: usize) -> usize {
        let chain_output_size = self.chain.compute_output_size(input_size);
        return self.demod.compute_output_size(chain_output_size);
    }

    fn set_input_size(&mut self, input_size: usize) {
        let chain_output_size = self.chain.compute_output_size(input_size);
        self.chain.set_input_size(input_size);
        self.demod.set_input_size(chain_output_size);
        self.buffer = AlignedVec::new_zeroed(chain_output_size);
    }
}
