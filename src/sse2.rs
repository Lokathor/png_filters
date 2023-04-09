#![cfg(any(target_arch = "x86", target_arch = "x86_64"))]

//! PNG filter functions specialized to the `sse2` cpu extension.

#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[inline]
fn m128i_as_mut_slice(m: &mut __m128i) -> &mut [u8] {
  let data = m as *mut __m128i as *mut u8;
  let len = core::mem::size_of::<__m128i>();
  unsafe { core::slice::from_raw_parts_mut(data, len) }
}

/// Like [`recon_sub_fallback`], but specialized to `sse2`.
///
/// ## Safety
/// * The `sse2` CPU feature must be available at runtime.
#[target_feature(enable = "sse2")]
pub unsafe fn recon_sub<const BYTES_PER_PIXEL: usize>(filtered_row: &mut [u8]) {
  assert!(BYTES_PER_PIXEL <= 8);
  debug_assert_eq!(filtered_row.len() % BYTES_PER_PIXEL, 0);
  //
  let mut a: __m128i = unsafe { core::mem::zeroed() };
  filtered_row.chunks_exact_mut(BYTES_PER_PIXEL).for_each(|chunk| {
    let mut x: __m128i = unsafe { core::mem::zeroed() };
    m128i_as_mut_slice(&mut x)[..BYTES_PER_PIXEL].copy_from_slice(chunk);
    x = unsafe { _mm_add_epi8(x, a) };
    chunk.copy_from_slice(&m128i_as_mut_slice(&mut x)[..BYTES_PER_PIXEL]);
    a = x;
  })
}
