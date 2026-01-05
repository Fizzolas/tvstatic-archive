use image::{ImageBuffer, Rgb};
use nalgebra::{DMatrix, DVector, Matrix3};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WarpError {
    #[error("not enough points")]
    NotEnoughPoints,
    #[error("singular solve")]
    Singular,
}

#[derive(Debug, Clone, Copy)]
pub struct Pt2 {
    pub x: f64,
    pub y: f64,
}

/// Compute a homography H mapping src -> dst from exactly four point correspondences.
///
/// This implements a Direct Linear Transform-style solve for 8 parameters (h33 fixed to 1).
/// Conceptually matches common homography/warpPerspective pipelines described in OpenCV docs. [web:215]
pub fn homography_from_4(src: [Pt2; 4], dst: [Pt2; 4]) -> Result<Matrix3<f64>, WarpError> {
    // Unknowns: h11 h12 h13 h21 h22 h23 h31 h32 (h33=1)
    let mut a = DMatrix::<f64>::zeros(8, 8);
    let mut b = DVector::<f64>::zeros(8);

    for i in 0..4 {
        let x = src[i].x;
        let y = src[i].y;
        let u = dst[i].x;
        let v = dst[i].y;

        // u = (h11 x + h12 y + h13) / (h31 x + h32 y + 1)
        // v = (h21 x + h22 y + h23) / (h31 x + h32 y + 1)
        // Rearranged:
        // h11 x + h12 y + h13 - u h31 x - u h32 y = u
        // h21 x + h22 y + h23 - v h31 x - v h32 y = v

        let r0 = i * 2;
        let r1 = r0 + 1;

        a[(r0, 0)] = x;
        a[(r0, 1)] = y;
        a[(r0, 2)] = 1.0;
        a[(r0, 6)] = -u * x;
        a[(r0, 7)] = -u * y;
        b[r0] = u;

        a[(r1, 3)] = x;
        a[(r1, 4)] = y;
        a[(r1, 5)] = 1.0;
        a[(r1, 6)] = -v * x;
        a[(r1, 7)] = -v * y;
        b[r1] = v;
    }

    let sol = a.lu().solve(&b).ok_or(WarpError::Singular)?;

    Ok(Matrix3::new(
        sol[0], sol[1], sol[2],
        sol[3], sol[4], sol[5],
        sol[6], sol[7], 1.0,
    ))
}

pub fn apply_h(h: &Matrix3<f64>, p: Pt2) -> Pt2 {
    let x = p.x;
    let y = p.y;
    let denom = h[(2, 0)] * x + h[(2, 1)] * y + h[(2, 2)];
    let u = (h[(0, 0)] * x + h[(0, 1)] * y + h[(0, 2)]) / denom;
    let v = (h[(1, 0)] * x + h[(1, 1)] * y + h[(1, 2)]) / denom;
    Pt2 { x: u, y: v }
}

/// Warp an RGB image using inverse mapping and nearest sampling.
///
/// For each destination pixel, compute source coordinate via H^{-1} and sample.
/// This is the standard inverse-mapping approach used in perspective warps. [web:258]
pub fn warp_perspective_nearest(
    src: &ImageBuffer<Rgb<u8>, Vec<u8>>,
    h_src_to_dst: &Matrix3<f64>,
    dst_w: u32,
    dst_h: u32,
) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, WarpError> {
    let h_inv = h_src_to_dst.try_inverse().ok_or(WarpError::Singular)?;
    let mut dst = ImageBuffer::new(dst_w, dst_h);

    let sw = src.width() as i32;
    let sh = src.height() as i32;

    for y in 0..dst_h {
        for x in 0..dst_w {
            let p = apply_h(&h_inv, Pt2 { x: x as f64, y: y as f64 });
            let sx = p.x.round() as i32;
            let sy = p.y.round() as i32;
            let px = if sx >= 0 && sx < sw && sy >= 0 && sy < sh {
                *src.get_pixel(sx as u32, sy as u32)
            } else {
                Rgb([0, 0, 0])
            };
            dst.put_pixel(x, y, px);
        }
    }

    Ok(dst)
}
