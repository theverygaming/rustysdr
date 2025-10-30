use crate::block::DspBlock;
use std::{boxed::Box, vec::Vec};
use volk_rs::vec::AlignedVec;

pub struct DspChain<T> {
    blocks: Vec<Box<dyn DspBlock<T>>>,
    buffer1: AlignedVec<T>,
    buffer2: AlignedVec<T>,
}

impl<T: Clone> DspChain<T> {
    pub fn new() -> DspChain<T> {
        DspChain {
            blocks: Vec::new(),
            buffer1: AlignedVec::new_zeroed(1048576),
            buffer2: AlignedVec::new_zeroed(1048576),
        }
    }

    pub fn add_block(&mut self, block: Box<dyn DspBlock<T>>) {
        self.blocks.push(block);
    }
}

impl<T: Clone> DspBlock<T> for DspChain<T> {
    fn process(&mut self, input: &[T], output: &mut [T]) {
        let mut buf_in = &mut self.buffer1;
        let mut buf_out = &mut self.buffer2;
        buf_in[0..input.len()].clone_from_slice(input);

        let mut last_out_size = input.len();

        for block in &mut self.blocks {
            let out_size = block.compute_output_size(last_out_size);
            block.process(&mut buf_in[0..last_out_size], &mut buf_out[0..out_size]);
            last_out_size = out_size;
            let buf_in_tmp = buf_in;
            buf_in = buf_out;
            buf_out = buf_in_tmp;
        }
        assert!(output.len() == last_out_size, "invalid output length");
        output.clone_from_slice(&buf_in[0..last_out_size]); // buf_in since they were swapped
    }

    fn compute_output_size(&mut self, input_size: usize) -> usize {
        let mut last_out_size = input_size;
        for block in &mut self.blocks {
            let out_size = block.compute_output_size(last_out_size);
            last_out_size = out_size;
        }
        last_out_size
    }

    fn set_input_size(&mut self, input_size: usize) {
        let mut max_out_size = input_size;
        let mut last_out_size = input_size;
        for block in &mut self.blocks {
            block.set_input_size(last_out_size);
            let out_size = block.compute_output_size(last_out_size);
            last_out_size = out_size;
            max_out_size = std::cmp::max(max_out_size, last_out_size);
        }
        self.buffer1 = AlignedVec::new_zeroed(max_out_size);
        self.buffer2 = AlignedVec::new_zeroed(max_out_size);
    }
}
