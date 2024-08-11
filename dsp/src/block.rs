use volk_rs::vec::AlignedVec;
// TODO: function that returns name for block & info about it's parameters so a graph of them can be generated
// admin interface -> view DSP chain?


pub trait DspBlock<T> {
    fn process(&mut self, input: &mut [T], output: &mut [T]);
    fn compute_output_size(&mut self, input_size: usize) -> usize;
    fn set_input_size(&mut self, input_size: usize);
}

pub trait DspBlockConv<Tin, Tout> {
    fn process(&mut self, input: &mut [Tin], output: &mut [Tout]);
}

pub trait DspSource<T> {
    fn process(&mut self, output: &mut [T]);
}

pub trait DspSink<T> {
    fn process(&mut self, input: &[T]);
}


/*pub trait Block<T_IN, T_OUT> {

}

pub struct BlockImpl<T_IN, T_OUT> {

}*/
