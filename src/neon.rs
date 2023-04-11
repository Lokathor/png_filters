#![cfg(target_arch = "aarch64")]

//! PNG filter functions specialized to the `neon` cpu extension.

use core::arch::aarch64::*;

#[inline]
#[must_use]
fn uint8x8_t_as_mut_slice(m: &mut uint8x8_t) -> &mut [u8] {
  let data = m as *mut uint8x8_t as *mut u8;
  let len = core::mem::size_of::<uint8x8_t>();
  unsafe { core::slice::from_raw_parts_mut(data, len) }
}

#[inline]
#[must_use]
fn int16x8_t_as_mut_slice(m: &mut int16x8_t) -> &mut [i16] {
  let data = m as *mut int16x8_t as *mut i16;
  let len = core::mem::size_of::<int16x8_t>();
  unsafe { core::slice::from_raw_parts_mut(data, len) }
}

#[inline]
#[target_feature(enable = "neon")]
unsafe fn uint8x8_t_load<const BYTES_PER_PIXEL: usize>(chunk: &[u8]) -> uint8x8_t {
  if BYTES_PER_PIXEL == 8 {
    vld1_u8(chunk.as_ptr())
  } else {
    let mut x: uint8x8_t = unsafe { core::mem::zeroed() };
    uint8x8_t_as_mut_slice(&mut x)[..BYTES_PER_PIXEL].copy_from_slice(chunk);
    x
  }
}
#[inline]
#[target_feature(enable = "neon")]
unsafe fn uint8x8_t_store<const BYTES_PER_PIXEL: usize>(chunk: &mut [u8], mut x: uint8x8_t) {
  if BYTES_PER_PIXEL == 8 {
    vst1_u8(chunk.as_mut_ptr(), x)
  } else {
    chunk.copy_from_slice(&uint8x8_t_as_mut_slice(&mut x)[..BYTES_PER_PIXEL]);
  }
}

/// Like [`recon_sub_fallback`](super::recon_sub_fallback), but specialized to
/// `neon`.
///
/// ## Safety
/// * The `neon` CPU feature must be available at runtime.
#[target_feature(enable = "neon")]
pub unsafe fn recon_sub<const BYTES_PER_PIXEL: usize>(filtered_row: &mut [u8]) {
  assert!(BYTES_PER_PIXEL <= 8);
  debug_assert_eq!(filtered_row.len() % BYTES_PER_PIXEL, 0);
  //
  let mut a: uint8x8_t = unsafe { core::mem::zeroed() };
  filtered_row.chunks_exact_mut(BYTES_PER_PIXEL).for_each(|chunk| {
    let mut x: uint8x8_t = uint8x8_t_load::<BYTES_PER_PIXEL>(chunk);
    x = unsafe { vadd_u8(x, a) };
    uint8x8_t_store::<BYTES_PER_PIXEL>(chunk, x);
    a = x;
  })
}

/// Like [`recon_up_fallback`](super::recon_up_fallback), but specialized to
/// `neon`.
///
/// ## Safety
/// * The `neon` CPU feature must be available at runtime.
#[target_feature(enable = "neon")]
pub unsafe fn recon_up(filtered_row: &mut [u8], previous_row: &[u8]) {
  debug_assert_eq!(filtered_row.len(), previous_row.len());
  //
  filtered_row.iter_mut().zip(previous_row.iter()).for_each(|(x, b)| *x = x.wrapping_add(*b))
}

/// Like [`recon_average_fallback`](super::recon_average_fallback), but
/// specialized to `neon`.
///
/// ## Safety
/// * The `neon` CPU feature must be available at runtime.
#[target_feature(enable = "neon")]
pub unsafe fn recon_average<const BYTES_PER_PIXEL: usize>(
  filtered_row: &mut [u8], previous_row: &[u8],
) {
  assert!(BYTES_PER_PIXEL <= 8);
  debug_assert_eq!(filtered_row.len() % BYTES_PER_PIXEL, 0);
  debug_assert_eq!(filtered_row.len(), previous_row.len());
  //
  // Recon(x) = Filt(x) + floor((Recon(a) + Recon(b)) / 2)
  //
  // * (a + b)/2 has to be done with 9-bit precision
  // * x + ave is done with u8_wrapping
  //
  let mut a: uint8x8_t = unsafe { core::mem::zeroed() };
  filtered_row
    .chunks_exact_mut(BYTES_PER_PIXEL)
    .zip(previous_row.chunks_exact(BYTES_PER_PIXEL))
    .for_each(|(x_chunk, b_chunk)| {
      let mut x: uint8x8_t = uint8x8_t_load::<BYTES_PER_PIXEL>(x_chunk);
      let b: uint8x8_t = uint8x8_t_load::<BYTES_PER_PIXEL>(b_chunk);
      {
        let ab_half = vhadd_u8(a, b);
        x = unsafe { vadd_u8(x, ab_half) };
      }
      uint8x8_t_store::<BYTES_PER_PIXEL>(x_chunk, x);
      a = x;
    })
}

/// As [`recon_average_top_fallback`](super::recon_average_top_fallback), but
/// specialized to `neon`.
///
/// ## Safety
/// * The `neon` CPU feature must be available at runtime.
#[target_feature(enable = "neon")]
pub unsafe fn recon_average_top<const BYTES_PER_PIXEL: usize>(filtered_row: &mut [u8]) {
  assert!(BYTES_PER_PIXEL <= 8);
  debug_assert_eq!(filtered_row.len() % BYTES_PER_PIXEL, 0);
  //
  let mut a: uint8x8_t = unsafe { core::mem::zeroed() };
  filtered_row.chunks_exact_mut(BYTES_PER_PIXEL).for_each(|chunk| {
    let mut x: uint8x8_t = uint8x8_t_load::<BYTES_PER_PIXEL>(chunk);
    x = unsafe { vadd_u8(x, vshr_n_u8::<1>(a)) };
    uint8x8_t_store::<BYTES_PER_PIXEL>(chunk, x);
    a = x;
  })
}

/// As [`recon_paeth_fallback`](super::recon_paeth_fallback), but
/// specialized to `neon`.
///
/// ## Safety
/// * The `neon` CPU feature must be available at runtime.
#[target_feature(enable = "neon")]
pub unsafe fn recon_paeth<const BYTES_PER_PIXEL: usize>(
  filtered_row: &mut [u8], previous_row: &[u8],
) {
  assert!(BYTES_PER_PIXEL <= 8);
  debug_assert_eq!(filtered_row.len() % BYTES_PER_PIXEL, 0);
  debug_assert_eq!(filtered_row.len(), previous_row.len());
  //
  let mut a: int16x8_t = unsafe { core::mem::zeroed() };
  let mut c: int16x8_t = unsafe { core::mem::zeroed() };
  filtered_row
    .chunks_exact_mut(BYTES_PER_PIXEL)
    .zip(previous_row.chunks_exact(BYTES_PER_PIXEL))
    .for_each(|(x_chunk, b_chunk)| {
      let mut x: uint8x8_t = uint8x8_t_load::<BYTES_PER_PIXEL>(x_chunk);
      let mut b: int16x8_t = unsafe { core::mem::zeroed() }; // i16
      int16x8_t_as_mut_slice(&mut b)
        .iter_mut()
        .zip(b_chunk.iter())
        .for_each(|(j, k)| *j = *k as i16);
      {
        let p = vsubq_s16(vaddq_s16(a, b), c);
        let pa = vabsq_s16(vsubq_s16(p, a));
        let pb = vabsq_s16(vsubq_s16(p, b));
        let pc = vabsq_s16(vsubq_s16(p, c));
        let pa_le_pb = vcleq_s16(pa, pb);
        let pa_le_pc = vcleq_s16(pa, pc);
        let pa_le_pb_and_pa_le_pc = vandq_u16(pa_le_pb, pa_le_pc);
        let pb_le_pc = vcleq_s16(pb, pc);
        let pick_b_or_c = vbslq_s16(pb_le_pc, b, c);
        let paeth_s16: int16x8_t = vbslq_s16(pa_le_pb_and_pa_le_pc, a, pick_b_or_c);
        let paeth_u8: uint8x16_t = vreinterpretq_u8_s16(paeth_s16);
        let paeth: uint8x8_t = vget_low_u8(vuzp1q_u8(paeth_u8, unsafe { core::mem::zeroed() }));
        x = vadd_u8(x, paeth);
      }
      uint8x8_t_store::<BYTES_PER_PIXEL>(x_chunk, x);
      let wide_x: uint8x16_t = vcombine_u8(x, unsafe { core::mem::zeroed() });
      let zipped_x: uint8x16_t = vzip1q_u8(wide_x, unsafe { core::mem::zeroed() });
      a = vreinterpretq_s16_u8(zipped_x);
      c = b;
    })
}
