fn cosine(params: &[f32], output: &mut [f32]) {
    let pi = std::f32::consts::PI;
    let n2 = output.len() as f32;
    for i in 0..output.len() {
        let n = i as f32;
        output[i] = params[0] - params[1] * f32::cos((2.0*pi*n)/n2) + params[2] * f32::cos((4.0*pi*n)/n2) - params[3] * f32::cos((6.0*pi*n)/n2);
    }
}

pub fn nuttall(output: &mut [f32]) {
    cosine(&[0.355768, 0.487396, 0.144232, 0.012604], output);
}

pub fn rectangular(output: &mut [f32]) {
    for i in 0..output.len() {
        output[i] = 1.0;
    }
}

pub fn blackmanharris(output: &mut [f32]) {
    cosine(&[0.35875, 0.48829, 0.14128, 0.01168], output);
}
