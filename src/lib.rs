#![feature(target_feature_11)]

//! Functions to perform the [PNG Filters][png-filters].
//!
//! [png-filters]: https://www.w3.org/TR/png/#9Filter-types
//!
//! The PNG spec defines filter *method* 0, which includes five different filter
//! *types*. When you "filter" the data that's applying a filter, when you
//! "reconstruct" the data that's removing the filter.
//!
//! To to either filter or reconstruct a pixel `x` you need the value for one or
//! more nearby pixels, they're named like this:
//!
//! ```txt
//! c b
//! a x
//! ```
//!
//! * `a` is the pixel to the left.
//! * `b` is the pixel above.
//! * `c` is the pixel above the pixel to the left.
//! * Whenever a pixel would be out of bounds of the image, use 0.
//!
//! Depending on the PNG's color format and bit depth, the pixels will each be 1
//! to 8 bytes. You must apply the filter or reconstruction operation *per byte*
//! within the pixel. This means that applying and removing filters can often
//! benefit greatly from using SIMD operations. There is, however, some overhead
//! involved to get the data into and out of SIMD registers, so when the number
//! of bytes per pixel is too low the data transfer overhead defeats the speed
//! gains of the SIMD operation itself.

#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn m128i_as_mut_slice(m: &mut __m128i) -> &mut [u8] {
  let data = m as *mut __m128i as *mut u8;
  let len = core::mem::size_of::<__m128i>();
  unsafe { core::slice::from_raw_parts_mut(data, len) }
}

/// `Recon(x) = Filt(x) + Recon(a)`
///
/// This version doesn't use any special intrinsics, and so it can always be
/// called regardless of `target_arch` or specific CPU feature availability.
#[inline]
pub fn recon_sub_fallback<const BYTES_PER_PIXEL: usize>(filtered_row: &mut [u8]) {
  assert!(BYTES_PER_PIXEL <= 8);
  debug_assert_eq!(filtered_row.len() % BYTES_PER_PIXEL, 0);
  //
  let mut a: [u8; BYTES_PER_PIXEL] = [0; BYTES_PER_PIXEL];
  filtered_row.chunks_exact_mut(BYTES_PER_PIXEL).for_each(|chunk| {
    let mut x: [u8; BYTES_PER_PIXEL] = chunk.try_into().unwrap();
    x.iter_mut().zip(a.iter()).for_each(|(x, a)| *x = x.wrapping_add(*a));
    chunk.copy_from_slice(&x);
    a = x;
  })
}

/// Like `recon_sub_fallback`, but specialized to `sse2`.
#[target_feature(enable = "sse2")]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn recon_sub_sse2<const BYTES_PER_PIXEL: usize>(filtered_row: &mut [u8]) {
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

// TODO: recon_sub_neon

// TODO: up

// TODO: average

// TODO: paeth

// TODO: versions that apply the filter
