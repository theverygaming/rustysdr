use volk_rs::{vec::AlignedVec, Complex};

#[test]
fn volk_works() {
    let input: AlignedVec<core::ffi::c_short> = AlignedVec::from_elem(1, 500);
    let mut taps: AlignedVec<Complex<f32>> = AlignedVec::from_elem(Complex { re: 5.0, im: 2.0 }, 500);
    let mut result: Complex<f32> = Complex { re: 0.0, im: 0.0 };
    volk_rs::kernels::volk_16i_32fc_dot_prod_32fc(&input, &mut result, &mut taps);
    assert!(result.re != 0.0 && result.im != 0.0, "borked");
}
