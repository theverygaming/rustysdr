use std::{vec::Vec, boxed::Box};
use volk_rs::vec::AlignedVec;
use crate::block::DspBlock;

pub struct DspChain<T> {
    blocks: Vec<Box<dyn DspBlock<T>>>,
    buffer1: AlignedVec<T>,
    buffer2: AlignedVec<T>,
}

impl<T: Clone> DspChain<T> {
    pub fn new() -> DspChain<T> {
        DspChain {
            blocks: Vec::new(),
            buffer1: AlignedVec::new_zeroed(1000000),
            buffer2: AlignedVec::new_zeroed(1000000),
        }
    }

    pub fn add_block(&mut self, block: Box<dyn DspBlock<T>>) {
        self.blocks.push(block);
    }
}

impl<T: Clone> DspBlock<T> for DspChain<T> {
    fn process(&mut self, input: &mut [T], output: &mut [T]) {
        // horribly inefficient but works:tm: (two extra unnecessary copies made), should definitely be improved
        let mut buf_in = &mut self.buffer1[0 .. input.len()];
        let mut buf_out = &mut self.buffer2[0 .. input.len()];
        buf_in.clone_from_slice(input);
        for block in &mut self.blocks {
            block.process(buf_in, buf_out);
            let buf_in_tmp = buf_in;
            buf_in = buf_out;
            buf_out = buf_in_tmp;
        }
        output.clone_from_slice(buf_in); // buf_in since they were swapped
    }
}
