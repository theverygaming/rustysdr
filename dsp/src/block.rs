// TODO: function that returns name for block & info about it's parameters so a graph of them can be generated
// admin interface -> view DSP chain?


pub trait DspBlock<T> {
    fn process(&mut self, input: &mut [T], output: &mut [T]);
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
