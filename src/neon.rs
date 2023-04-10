#![cfg(target_arch = "aarch64")]

//! PNG filter functions specialized to the `neon` cpu extension.

use core::arch::aarch64::*;

#[inline]
#[must_use]
fn int8x8_t_as_mut_slice(m: &mut int8x8_t) -> &mut [u8] {
  let data = m as *mut int8x8_t as *mut u8;
  let len = core::mem::size_of::<int8x8_t>();
  unsafe { core::slice::from_raw_parts_mut(data, len) }
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
  let mut a: int8x8_t = unsafe { core::mem::zeroed() };
  filtered_row.chunks_exact_mut(BYTES_PER_PIXEL).for_each(|chunk| {
    let mut x: int8x8_t = unsafe { core::mem::zeroed() };
    int8x8_t_as_mut_slice(&mut x)[..BYTES_PER_PIXEL].copy_from_slice(chunk);
    x = unsafe { vadd_s8(x, a) };
    chunk.copy_from_slice(&int8x8_t_as_mut_slice(&mut x)[..BYTES_PER_PIXEL]);
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
  // * (a + b)/2 has to be done with 16-bit precision
  // * x + ave is done with u8_wrapping
  //
  let mut a: int8x8_t = unsafe { core::mem::zeroed() };
  filtered_row
    .chunks_exact_mut(BYTES_PER_PIXEL)
    .zip(previous_row.chunks_exact(BYTES_PER_PIXEL))
    .for_each(|(x_chunk, b_chunk)| {
      let mut x: int8x8_t = unsafe { core::mem::zeroed() };
      int8x8_t_as_mut_slice(&mut x)[..BYTES_PER_PIXEL].copy_from_slice(x_chunk);
      let mut b: int8x8_t = unsafe { core::mem::zeroed() };
      int8x8_t_as_mut_slice(&mut b)[..BYTES_PER_PIXEL].copy_from_slice(b_chunk);
      {
        let ab_half = vhadd_s8(a, b);
        x = unsafe { vadd_s8(x, ab_half) };
      }
      x_chunk.copy_from_slice(&int8x8_t_as_mut_slice(&mut x)[..BYTES_PER_PIXEL]);
      a = x;
    })
}
