use volk_rs::{types::complex, vec::AlignedVec};

#[test]
fn volk_works() {
    let mut input: AlignedVec<core::ffi::c_short> = AlignedVec::from_elem(1, 500);
    let mut taps: AlignedVec<complex<f32>> = AlignedVec::from_elem(complex { r: 5.0, i: 2.0 }, 500);
    let mut result: complex<f32> = complex { r: 0.0, i: 0.0 };
    volk_rs::kernels::volk_16i_32fc_dot_prod_32fc(&mut result, &mut input, &mut taps);
    assert!(result.r != 0.0 && result.i != 0.0, "borked");
}
