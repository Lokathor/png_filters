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

use core::slice::ChunksExactMut;

pub mod fallbacks;
#[cfg(target_arch = "aarch64")]
pub mod neon;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod sse2;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod sse4_1;

/// Given the bytes for each filtered line, unfilters the data in place.
///
/// On each line, the first byte of the line will be the filter type, and the
/// following bytes will be the image data. The number of following bytes should
/// evenly divide by `BYTES_PER_PIXEL`.
#[inline]
pub fn unfilter_lines<const BYTES_PER_PIXEL: usize>(lines: ChunksExactMut<'_, u8>) {
  let mut sub: unsafe fn(&mut [u8]) = fallbacks::recon_sub::<BYTES_PER_PIXEL>;
  let mut up: unsafe fn(&mut [u8], &[u8]) = fallbacks::recon_up;
  let mut average: unsafe fn(&mut [u8], &[u8]) = fallbacks::recon_average::<BYTES_PER_PIXEL>;
  let mut average_top: unsafe fn(&mut [u8]) = fallbacks::recon_average_top::<BYTES_PER_PIXEL>;
  let mut paeth: unsafe fn(&mut [u8], &[u8]) = fallbacks::recon_paeth::<BYTES_PER_PIXEL>;

  #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
  {
    let has_sse4_1 = is_x86_feature_detected!("sse4.1");
    let has_sse2 = is_x86_feature_detected!("sse2");
    if BYTES_PER_PIXEL >= 3 && has_sse4_1 {
      paeth = sse4_1::recon_paeth::<BYTES_PER_PIXEL>;
    } else if BYTES_PER_PIXEL >= 3 && has_sse2 {
      paeth = sse2::recon_paeth::<BYTES_PER_PIXEL>;
    }
    if BYTES_PER_PIXEL >= 8 && has_sse2 {
      sub = sse2::recon_sub::<BYTES_PER_PIXEL>;
      average = sse2::recon_average::<BYTES_PER_PIXEL>;
      average_top = sse2::recon_average_top::<BYTES_PER_PIXEL>;
    }
  }
  #[cfg(target_arch = "aarch64")]
  {
    let has_neon = std::arch::is_aarch64_feature_detected!("neon");
    if has_neon {
      up = neon::recon_up;
    }
    if (BYTES_PER_PIXEL == 2 || BYTES_PER_PIXEL >= 4) && has_neon {
      paeth = neon::recon_paeth::<BYTES_PER_PIXEL>;
    }
    if BYTES_PER_PIXEL >= 4 && has_neon {
      sub = neon::recon_sub::<BYTES_PER_PIXEL>;
      average = neon::recon_average::<BYTES_PER_PIXEL>;
      average_top = neon::recon_average_top::<BYTES_PER_PIXEL>;
    }
  }

  // Won't panic: `chunk_size` is always non-zero (ChunksExactMut invariant).
  let mut lines = lines.map(|line| line.split_first_mut().unwrap());

  // most filters run differently or not at all on the top line.
  let mut previous: &[u8] = if let Some((filter, line)) = lines.next() {
    match filter {
      1 => unsafe { sub(line) },
      2 => (),
      3 => unsafe { average_top(line) },
      4 => (),
      _ => (),
    }
    *filter = 0;
    line
  } else {
    return;
  };

  // now handle all other lines
  lines.for_each(|(filter, line)| {
    match filter {
      1 => unsafe { sub(line) },
      2 => unsafe { up(line, previous) },
      3 => unsafe { average(line, previous) },
      4 => unsafe { paeth(line, previous) },
      _ => (),
    }
    *filter = 0;
    previous = line;
  });
}
