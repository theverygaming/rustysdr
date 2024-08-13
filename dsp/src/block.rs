use crate::stream::Stream;
use std::sync::Arc;
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

pub trait Block<TIn, TOut> {
    // public methods
    fn get_input(&mut self) -> Option<Arc<Stream<TIn>>>;
    fn get_output(&mut self) -> Option<Arc<Stream<TOut>>>;
    fn start(&mut self);
    fn stop(&mut self);
}

#[macro_export]
macro_rules! impl_block{
    ($blockname:ident, $traitname:ident, $($body:item),* $(,)?)=>{
        trait $traitname {
            fn run(&self) -> bool;
        }

        impl<T: 'static> Block<T, T> for Arc<$blockname<T>>
        where
            $blockname<T>: $traitname,
            T: Send,
        {
            $($body)*

            fn start(&mut self) {
                let clone = self.clone();
                *self.thread_handle.lock().unwrap() = Some(thread::spawn(move || {
                    loop {
                        if !clone.run() {
                            break;
                        }
                    }
                }));
            }

            fn stop(&mut self) {
                self.thread_handle.lock().unwrap().take().expect("thread must be running to be stopped").join().unwrap();
            }
        }
    }
}
