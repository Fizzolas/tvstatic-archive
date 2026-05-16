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
pub fn homography_from_4(src: [Pt2; 4], dst: [Pt2; 4]) -> Result<Matrix3<f64>, WarpError> {
    let mut a = DMatrix::<f64>::zeros(8, 8);
    let mut b = DVector::<f64>::zeros(8);

    for i in 0..4 {
        let x = src[i].x;
        let y = src[i].y;
        let u = dst[i].x;
        let v = dst[i].y;

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

/// Warp an RGB image using inverse mapping with **bilinear interpolation**.
///
/// For each destination pixel, the source coordinate is computed via H^{-1}.
/// Rather than rounding to nearest (which causes aliasing on camera-captured frames),
/// we blend the four surrounding source pixels by their fractional distance.
/// This significantly reduces colour quantization errors in the scan profile before
/// the palette classifier runs.
///
/// Previously named `warp_perspective_nearest` — renamed to match actual behaviour.
pub fn warp_perspective_bilinear(
    src: &ImageBuffer<Rgb<u8>, Vec<u8>>,
    h_src_to_dst: &Matrix3<f64>,
    dst_w: u32,
    dst_h: u32,
) -> Result<ImageBuffer<Rgb<u8>, Vec<u8>>, WarpError> {
    let h_inv = h_src_to_dst.try_inverse().ok_or(WarpError::Singular)?;
    let mut dst = ImageBuffer::new(dst_w, dst_h);

    let sw = src.width() as i32;
    let sh = src.height() as i32;

    for dy in 0..dst_h {
        for dx in 0..dst_w {
            let p = apply_h(&h_inv, Pt2 { x: dx as f64, y: dy as f64 });
            let px = sample_bilinear(src, p.x, p.y, sw, sh);
            dst.put_pixel(dx, dy, px);
        }
    }

    Ok(dst)
}

/// Bilinear sample of an RGB image at sub-pixel (sx, sy).
///
/// Clamps to image bounds so no out-of-range access occurs.
/// Falls back to black only if the source coordinate is entirely outside the image.
#[inline]
fn sample_bilinear(src: &ImageBuffer<Rgb<u8>, Vec<u8>>, sx: f64, sy: f64, sw: i32, sh: i32) -> Rgb<u8> {
    // Integer floor coordinates of the top-left corner
    let x0 = sx.floor() as i32;
    let y0 = sy.floor() as i32;

    // Quick reject: entirely outside image
    if x0 < -1 || y0 < -1 || x0 >= sw || y0 >= sh {
        return Rgb([0, 0, 0]);
    }

    // Fractional parts
    let fx = (sx - sx.floor()) as f32;
    let fy = (sy - sy.floor()) as f32;

    // Clamp neighbours to valid range
    let x1 = (x0 + 1).clamp(0, sw - 1) as u32;
    let y1 = (y0 + 1).clamp(0, sh - 1) as u32;
    let x0c = x0.clamp(0, sw - 1) as u32;
    let y0c = y0.clamp(0, sh - 1) as u32;

    let p00 = src.get_pixel(x0c, y0c).0;
    let p10 = src.get_pixel(x1,  y0c).0;
    let p01 = src.get_pixel(x0c, y1 ).0;
    let p11 = src.get_pixel(x1,  y1 ).0;

    // Lerp each channel independently
    let lerp_ch = |c00: u8, c10: u8, c01: u8, c11: u8| -> u8 {
        let top    = c00 as f32 * (1.0 - fx) + c10 as f32 * fx;
        let bottom = c01 as f32 * (1.0 - fx) + c11 as f32 * fx;
        (top * (1.0 - fy) + bottom * fy).round() as u8
    };

    Rgb([
        lerp_ch(p00[0], p10[0], p01[0], p11[0]),
        lerp_ch(p00[1], p10[1], p01[1], p11[1]),
        lerp_ch(p00[2], p10[2], p01[2], p11[2]),
    ])
}
