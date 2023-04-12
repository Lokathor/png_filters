#![cfg(any(target_arch = "x86", target_arch = "x86_64"))]

//! PNG filter functions specialized to the `sse2` cpu extension.

#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

const ZEROED: __m128i = unsafe { core::mem::transmute([0_u64; 2]) };

#[inline]
fn m128i_as_mut_u8s(m: &mut __m128i) -> &mut [u8] {
  let data = m as *mut __m128i as *mut u8;
  let len = core::mem::size_of::<__m128i>() / core::mem::size_of::<u8>();
  unsafe { core::slice::from_raw_parts_mut(data, len) }
}
#[inline]
fn m128i_as_mut_i16s(m: &mut __m128i) -> &mut [i16] {
  let data = m as *mut __m128i as *mut i16;
  let len = core::mem::size_of::<__m128i>() / core::mem::size_of::<i16>();
  unsafe { core::slice::from_raw_parts_mut(data, len) }
}

/// We always have to emulate this
#[inline]
#[target_feature(enable = "sse2")]
unsafe fn i16_le_sse2(a: __m128i, b: __m128i) -> __m128i {
  let lt = _mm_cmplt_epi16(a, b);
  let eq = _mm_cmpeq_epi16(a, b);
  _mm_or_si128(lt, eq)
}

/// Like [`recon_sub_fallback`](super::recon_sub_fallback), but specialized to
/// `sse2`.
///
/// ## Safety
/// * The `sse2` CPU feature must be available at runtime.
#[target_feature(enable = "sse4.1")]
pub unsafe fn recon_sub<const BYTES_PER_PIXEL: usize>(filtered_row: &mut [u8]) {
  assert!(BYTES_PER_PIXEL <= 8);
  debug_assert_eq!(filtered_row.len() % BYTES_PER_PIXEL, 0);
  //
  let mut a: __m128i = ZEROED;
  filtered_row.chunks_exact_mut(BYTES_PER_PIXEL).for_each(|chunk| {
    let mut x: __m128i = ZEROED;
    m128i_as_mut_u8s(&mut x)[..BYTES_PER_PIXEL].copy_from_slice(chunk);
    x = unsafe { _mm_add_epi8(x, a) };
    chunk.copy_from_slice(&m128i_as_mut_u8s(&mut x)[..BYTES_PER_PIXEL]);
    a = x;
  })
}

/// Like [`recon_up_fallback`](super::recon_up_fallback), but specialized to
/// `sse2`.
///
/// You normally don't need to use this function. On Rust's `x86_64` and `i686`
/// targets the `sse2` feature is enabled by default, and the auto-vectorizer is
/// able to SIMD accelerate the normal fallback function quite well, so you can
/// just use that. You only need to call this version with `i585` targets when
/// you've dynamically detected that `sse2` is available.
///
/// ## Safety
/// * The `sse2` CPU feature must be available at runtime.
#[target_feature(enable = "sse4.1")]
pub unsafe fn recon_up(filtered_row: &mut [u8], previous_row: &[u8]) {
  debug_assert_eq!(filtered_row.len(), previous_row.len());
  //
  filtered_row.iter_mut().zip(previous_row.iter()).for_each(|(x, b)| *x = x.wrapping_add(*b))
}

/// Like [`recon_average_fallback`](super::recon_average_fallback), but
/// specialized to `sse2`.
///
/// ## Safety
/// * The `sse2` CPU feature must be available at runtime.
#[target_feature(enable = "sse2")]
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
  let mut a: __m128i = ZEROED; // i16
  filtered_row
    .chunks_exact_mut(BYTES_PER_PIXEL)
    .zip(previous_row.chunks_exact(BYTES_PER_PIXEL))
    .for_each(|(x_chunk, b_chunk)| {
      let mut x: __m128i = ZEROED; // u8
      m128i_as_mut_u8s(&mut x)[..BYTES_PER_PIXEL].copy_from_slice(x_chunk);
      let mut b: __m128i = ZEROED; // i16
      m128i_as_mut_i16s(&mut b).iter_mut().zip(b_chunk.iter()).for_each(|(j, k)| *j = *k as i16);
      {
        let average = _mm_srai_epi16(_mm_add_epi16(a, b), 1);
        let average_u8 = _mm_packus_epi16(average, ZEROED);
        x = _mm_add_epi8(x, average_u8);
      }
      x_chunk.copy_from_slice(&m128i_as_mut_u8s(&mut x)[..BYTES_PER_PIXEL]);
      a = _mm_unpacklo_epi8(x, ZEROED);
    })
}

/// Like [`recon_average_top_fallback`](super::recon_average_top_fallback), but
/// specialized to `sse2`.
///
/// ## Safety
/// * The `sse2` CPU feature must be available at runtime.
#[target_feature(enable = "sse4.1")]
pub unsafe fn recon_average_top<const BYTES_PER_PIXEL: usize>(filtered_row: &mut [u8]) {
  assert!(BYTES_PER_PIXEL <= 8);
  debug_assert_eq!(filtered_row.len() % BYTES_PER_PIXEL, 0);
  //
  // Recon(x) = Filt(x) + floor((Recon(a) + Recon(b)) / 2)
  //
  // * (a + b)/2 has to be done with 16-bit precision
  // * x + ave is done with u8_wrapping
  //
  let mut a: __m128i = ZEROED; // i16
  filtered_row.chunks_exact_mut(BYTES_PER_PIXEL).for_each(|chunk| {
    let mut x: __m128i = ZEROED; // u8
    m128i_as_mut_u8s(&mut x)[..BYTES_PER_PIXEL].copy_from_slice(chunk);
    {
      let half_a = _mm_srai_epi16(a, 1);
      let half_a_u8 = _mm_packus_epi16(half_a, ZEROED);
      x = _mm_add_epi8(x, half_a_u8);
    }
    chunk.copy_from_slice(&m128i_as_mut_u8s(&mut x)[..BYTES_PER_PIXEL]);
    a = _mm_unpacklo_epi8(x, ZEROED);
  })
}

/// Like [`recon_paeth_fallback`](super::recon_paeth_fallback), but specialized
/// to `sse2`.
///
/// ## Safety
/// * The `sse2` CPU feature must be available at runtime.
#[target_feature(enable = "sse4.1")]
pub unsafe fn recon_paeth<const BYTES_PER_PIXEL: usize>(
  filtered_row: &mut [u8], previous_row: &[u8],
) {
  assert!(BYTES_PER_PIXEL <= 8);
  debug_assert_eq!(filtered_row.len() % BYTES_PER_PIXEL, 0);
  debug_assert_eq!(filtered_row.len(), previous_row.len());
  //
  let mut a: __m128i = ZEROED; // i16
  let mut c: __m128i = ZEROED; // i16
  filtered_row
    .chunks_exact_mut(BYTES_PER_PIXEL)
    .zip(previous_row.chunks_exact(BYTES_PER_PIXEL))
    .for_each(|(x_chunk, b_chunk)| {
      let mut x: __m128i = ZEROED; // u8
      m128i_as_mut_u8s(&mut x)[..BYTES_PER_PIXEL].copy_from_slice(x_chunk);
      let mut b: __m128i = ZEROED; // i16
      m128i_as_mut_i16s(&mut b).iter_mut().zip(b_chunk.iter()).for_each(|(j, k)| *j = *k as i16);
      {
        let p = _mm_sub_epi16(_mm_add_epi16(a, b), c);
        let pa = _mm_abs_epi16(_mm_sub_epi16(p, a));
        let pb = _mm_abs_epi16(_mm_sub_epi16(p, b));
        let pc = _mm_abs_epi16(_mm_sub_epi16(p, c));
        let pa_le_pb = i16_le_sse2(pa, pb);
        let pa_le_pc = i16_le_sse2(pa, pc);
        let pa_le_pb_and_pa_le_pc = _mm_and_si128(pa_le_pb, pa_le_pc);
        let pb_le_pc = i16_le_sse2(pb, pc);
        let pick_b_or_c = _mm_blendv_epi8(c, b, pb_le_pc);
        let paeth16 = _mm_blendv_epi8(pick_b_or_c, a, pa_le_pb_and_pa_le_pc);
        let paeth = _mm_packus_epi16(paeth16, ZEROED);
        x = _mm_add_epi8(x, paeth);
      }
      x_chunk.copy_from_slice(&m128i_as_mut_u8s(&mut x)[..BYTES_PER_PIXEL]);
      a = _mm_unpacklo_epi8(x, ZEROED);
      c = b;
    })
}
