use crate::stream::Stream;
use std::sync::Arc;

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
    fn get_input(&mut self) -> Vec<Arc<Stream<TIn>>>;
    fn get_output(&mut self) -> Vec<Arc<Stream<TOut>>>;
    fn start(&mut self);
    fn stop(&mut self);
}

#[macro_export]
macro_rules! impl_block_methods {
    () => {
        fn start(&mut self) {
            for s in self.get_input() {
                s.start_reader();
            }
            for s in self.get_output() {
                s.start_writer();
            }
            let clone = self.clone();
            *self.thread_handle.lock().unwrap() = Some(thread::spawn(move || loop {
                if !clone.run() {
                    break;
                }
            }));
        }

        fn stop(&mut self) {
            for s in self.get_input() {
                s.stop_reader();
            }
            for s in self.get_output() {
                s.stop_writer();
            }
            self.thread_handle
                .lock()
                .unwrap()
                .take()
                .expect("thread must be running to be stopped")
                .join()
                .expect("worker thread panic");
        }
    };
}

#[macro_export]
macro_rules! impl_block_trait {
    ($traitname:ident) => {
        trait $traitname {
            fn run(&self) -> bool;
        }
    };
}

#[macro_export]
macro_rules! impl_block{
    ($blockname:ident, $traitname:ident, $($body:item),* $(,)?)=>{
        $crate::impl_block_trait!($traitname);

        impl<T: 'static> Block<T, T> for Arc<$blockname<T>>
        where
            $blockname<T>: $traitname,
            T: Send,
        {
            $($body)*

            $crate::impl_block_methods!();
        }
    };
}
