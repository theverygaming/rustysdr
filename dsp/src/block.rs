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
            fn run(&mut self) -> bool;
        }

        impl<T: 'static> Block<T, T> for Arc<Mutex<$blockname<T>>>
        where
            $blockname<T>: $traitname + Send,
        {
            $($body)*

            fn start(&mut self) {
                let clone = self.clone();
                self.lock().unwrap().thread_handle = Some(thread::spawn(move || {
                    loop {
                        let mut unlocked = clone.lock().unwrap();
                        if !unlocked.run() {
                            break;
                        }
                        drop(unlocked);
                    }
                }));
            }

            fn stop(&mut self) {
                self.lock().unwrap().thread_handle.take().expect("thread must be running to be stopped").join().unwrap();
            }
        }
    }
}
