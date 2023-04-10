#![warn(missing_docs)]

//! Functions to perform the [PNG Filters][png-filters].
//!
//! [png-filters]: https://www.w3.org/TR/png/#9Filter-types
//!
//! The PNG spec defines filter *method* 0, which includes five different filter
//! *types* 0 through 4. When you "filter" the data that's applying a filter,
//! when you "reconstruct" the data that's removing the filter.
//!
//! Filter type 0 has no effect on the data. The other four filter types
//! determine the bytes for pixel `x` based on the nearby pixels, named
//! according to this diagram:
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
//! Depending on the PNG's color format and bit depth, each pixel will be 1 to 8
//! bytes. Filtering operations are applied *per byte* within the pixel.
//!
//! Applying and removing filters can often benefit greatly from using SIMD
//! operations.
//!
//! * At the root of this crate are various "fallback" versions of the filters.
//!   These functions do not use any special intrinsics, and so they are always
//!   available to use safely.
//! * In the crate's sub-modules there are filter functions that take advantage
//!   of different CPU feature levels. With Aarch64 there's a single `neon`
//!   module, and with x86/64 there's one module per feature level.
//! * The functions use [target_feature][ref-tf] to enable the appropriate
//!   feature within their scope. This makes them `unsafe` to call, and you must
//!   check at runtime that the appropriate CPU feature is available (using
//!   either [`is_aarch64_feature_detected`][aarch64_detect] or
//!   [`is_x86_feature_detected`][x86_detect]).
//!
//! [aarch64_detect]:
//!     https://doc.rust-lang.org/stable/std/arch/macro.is_aarch64_feature_detected.html
//! [x86_detect]:
//!     https://doc.rust-lang.org/stable/std/arch/macro.is_x86_feature_detected.html
//! [ref-tf]:
//!     https://doc.rust-lang.org/reference/attributes/codegen.html#the-target_feature-attribute
//!
//! There is some overhead involved to get the data into and out of SIMD
//! registers. Generally, if the bytes per pixel is 1 or 2 then SIMD usage is
//! not advised. The extra time taken on memory operations will outweigh the
//! time saved on math operations.

#[cfg(target_arch = "aarch64")]
pub mod neon;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod sse2;

/// `Recon(x) = Filt(x) + Recon(a)`
///
/// This version doesn't use any special intrinsics, and so it can always be
/// called regardless of `target_arch` or specific CPU feature availability.
///
/// ## Panics
/// * `assert!(BYTES_PER_PIXEL <= 8);`
/// * `debug_assert_eq!(filtered_row.len() % BYTES_PER_PIXEL, 0);`
#[inline]
#[deny(unsafe_code)]
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

/// `Recon(x) = Filt(x) + Recon(b)`
///
/// This version doesn't use any special intrinsics, and so it can always be
/// called regardless of `target_arch` or specific CPU feature availability.
///
/// This function doesn't specialize on the bytes per pixel like most others do,
/// since it doesn't use the leftward pixel data.
///
/// This function does not have a "top" variant. There's no effect when doing a
/// wrapping add of 0.
///
/// ## Panic
/// * `debug_assert_eq!(filtered_row.len(), previous_row.len());`
#[inline]
#[deny(unsafe_code)]
pub fn recon_up_fallback(filtered_row: &mut [u8], previous_row: &[u8]) {
  debug_assert_eq!(filtered_row.len(), previous_row.len());
  //
  filtered_row.iter_mut().zip(previous_row.iter()).for_each(|(x, b)| *x = x.wrapping_add(*b))
}

/// `Recon(x) = Filt(x) + floor((Recon(a) + Recon(b)) / 2)`
///
/// This version doesn't use any special intrinsics, and so it can always be
/// called regardless of `target_arch` or specific CPU feature availability.
///
/// ## Panic
/// * `assert!(BYTES_PER_PIXEL <= 8);`
/// * `debug_assert_eq!(filtered_row.len() % BYTES_PER_PIXEL, 0);`
/// * `debug_assert_eq!(filtered_row.len(), previous_row.len());`
#[inline]
#[deny(unsafe_code)]
pub fn recon_average_fallback<const BYTES_PER_PIXEL: usize>(
  filtered_row: &mut [u8], previous_row: &[u8],
) {
  assert!(BYTES_PER_PIXEL <= 8);
  debug_assert_eq!(filtered_row.len() % BYTES_PER_PIXEL, 0);
  debug_assert_eq!(filtered_row.len(), previous_row.len());
  //
  let mut a: [u8; BYTES_PER_PIXEL] = [0; BYTES_PER_PIXEL];
  filtered_row
    .chunks_exact_mut(BYTES_PER_PIXEL)
    .zip(previous_row.chunks_exact(BYTES_PER_PIXEL))
    .for_each(|(x_chunk, b_chunk)| {
      let mut x: [u8; BYTES_PER_PIXEL] = x_chunk.try_into().unwrap();
      let b: [u8; BYTES_PER_PIXEL] = b_chunk.try_into().unwrap();
      x.iter_mut().zip(a.iter()).zip(b.iter()).for_each(|((x, a), b)| {
        let average: u8 = ((*a as i16 + *b as i16) / 2) as u8;
        *x = x.wrapping_add(average);
      });
      x_chunk.copy_from_slice(&x);
      a = x;
    })
}

/// As [`recon_average_fallback`], but for the top line of a PNG.
#[inline]
#[deny(unsafe_code)]
pub fn recon_average_top_fallback<const BYTES_PER_PIXEL: usize>(filtered_row: &mut [u8]) {
  assert!(BYTES_PER_PIXEL <= 8);
  debug_assert_eq!(filtered_row.len() % BYTES_PER_PIXEL, 0);
  //
  let mut a: [u8; BYTES_PER_PIXEL] = [0; BYTES_PER_PIXEL];
  filtered_row.chunks_exact_mut(BYTES_PER_PIXEL).for_each(|chunk| {
    let mut x: [u8; BYTES_PER_PIXEL] = chunk.try_into().unwrap();
    x.iter_mut().zip(a.iter()).for_each(|(x, a)| *x = x.wrapping_add(a / 2));
    chunk.copy_from_slice(&x);
    a = x;
  })
}

/// `Recon(x) = Filt(x) + PaethPredictor(Recon(a), Recon(b), Recon(c))`
///
/// This function does not have a "top" variant. If you inline the "previous"
/// row from the top (all zeroes) the paeth function always evaluates to zero,
/// thus causing the whole thing to be a no-op for the top row.
///
/// ## Panics
/// * `assert!(BYTES_PER_PIXEL <= 8);`
/// * `debug_assert_eq!(filtered_row.len() % BYTES_PER_PIXEL, 0);`
/// * `debug_assert_eq!(filtered_row.len(), previous_row.len());`
#[inline]
#[deny(unsafe_code)]
pub fn recon_paeth_fallback<const BYTES_PER_PIXEL: usize>(
  filtered_row: &mut [u8], previous_row: &[u8],
) {
  assert!(BYTES_PER_PIXEL <= 8);
  debug_assert_eq!(filtered_row.len() % BYTES_PER_PIXEL, 0);
  debug_assert_eq!(filtered_row.len(), previous_row.len());
  //
  let mut a: [u8; BYTES_PER_PIXEL] = [0; BYTES_PER_PIXEL];
  let mut c: [u8; BYTES_PER_PIXEL] = [0; BYTES_PER_PIXEL];
  filtered_row
    .chunks_exact_mut(BYTES_PER_PIXEL)
    .zip(previous_row.chunks_exact(BYTES_PER_PIXEL))
    .for_each(|(x_chunk, b_chunk)| {
      let mut x: [u8; BYTES_PER_PIXEL] = x_chunk.try_into().unwrap();
      let b: [u8; BYTES_PER_PIXEL] = b_chunk.try_into().unwrap();
      x.iter_mut().zip(a.iter()).zip(b.iter()).zip(c.iter()).for_each(|(((x, a), b), c)| {
        let p: i16 = *a as i16 + *b as i16 - *c as i16;
        let pa: i16 = (p - *a as i16).abs();
        let pb: i16 = (p - *b as i16).abs();
        let pc: i16 = (p - *c as i16).abs();
        *x = x.wrapping_add(if pa <= pb && pa <= pc {
          *a
        } else if pb <= pc {
          *b
        } else {
          *c
        });
      });
      x_chunk.copy_from_slice(&x);
      a = x;
      c = b;
    })
}
