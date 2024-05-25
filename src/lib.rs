#![warn(missing_docs)]

//! Functions to remove the [PNG Filters][png-filters] from encoded bytes.
//!
//! [png-filters]: https://www.w3.org/TR/png/#9Filter-types
//!
//! This crate has functions to "reconstruct" the bytes using plain rust and
//! also using explicit SIMD. There's some overhead to move data into and out of
//! SIMD registers, so at a low number of bytes per pixel you are advised to
//! *not* use the SIMD functions.
//!
//! Generally you should just call [`unfilter_lines`], which will handle an
//! entire image all at once, and it will automatically select the best
//! functions based on the bytes per pixel.

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
///
/// ## Panics
/// * `assert!(BYTES_PER_PIXEL <= 8);`
/// * `debug_assert_eq!(following_bytes.len() % BYTES_PER_PIXEL, 0);`
#[inline]
#[allow(unused_mut)]
pub fn unfilter_lines<const BYTES_PER_PIXEL: usize>(lines: ChunksExactMut<'_, u8>) {
  let mut sub: unsafe fn(&mut [u8]) = fallbacks::recon_sub::<BYTES_PER_PIXEL>;
  let mut up: unsafe fn(&mut [u8], &[u8]) = fallbacks::recon_up;
  let mut average: unsafe fn(&mut [u8], &[u8]) = fallbacks::recon_average::<BYTES_PER_PIXEL>;
  let mut average_top: unsafe fn(&mut [u8]) = fallbacks::recon_average_top::<BYTES_PER_PIXEL>;
  let mut paeth: unsafe fn(&mut [u8], &[u8]) = fallbacks::recon_paeth::<BYTES_PER_PIXEL>;

  #[cfg(FALSE)]
  if is_x86_feature_detected!("sse4.1") {
    sub = sse4_1::recon_sub::<BYTES_PER_PIXEL>;
    up = sse4_1::recon_up;
    average = sse4_1::recon_average::<BYTES_PER_PIXEL>;
    average_top = sse4_1::recon_average_top::<BYTES_PER_PIXEL>;
    paeth = sse4_1::recon_paeth::<BYTES_PER_PIXEL>;
  }
  #[cfg(FALSE)]
  if is_x86_feature_detected!("sse2") {
    sub = sse2::recon_sub::<BYTES_PER_PIXEL>;
    up = sse2::recon_up;
    average = sse2::recon_average::<BYTES_PER_PIXEL>;
    average_top = sse2::recon_average_top::<BYTES_PER_PIXEL>;
    paeth = sse2::recon_paeth::<BYTES_PER_PIXEL>;
  }
  #[cfg(FALSE)]
  if std::arch::is_aarch64_feature_detected!("neon") {
    sub = neon::recon_sub::<BYTES_PER_PIXEL>;
    up = neon::recon_up;
    average = neon::recon_average::<BYTES_PER_PIXEL>;
    average_top = neon::recon_average_top::<BYTES_PER_PIXEL>;
    paeth = neon::recon_paeth::<BYTES_PER_PIXEL>;
  }
  //#[cfg(FALSE)]
  #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
  {
    let has_sse4_1 = is_x86_feature_detected!("sse4.1");
    let has_sse2 = is_x86_feature_detected!("sse2");
    if BYTES_PER_PIXEL >= 8 && has_sse4_1 {
      average = sse4_1::recon_average::<BYTES_PER_PIXEL>;
      average_top = sse4_1::recon_average_top::<BYTES_PER_PIXEL>;
    } else if BYTES_PER_PIXEL >= 8 && has_sse2 {
      average = sse2::recon_average::<BYTES_PER_PIXEL>;
      average_top = sse2::recon_average_top::<BYTES_PER_PIXEL>;
    }
    if BYTES_PER_PIXEL >= 3 && has_sse4_1 {
      paeth = sse4_1::recon_paeth::<BYTES_PER_PIXEL>;
    } else if BYTES_PER_PIXEL >= 3 && has_sse2 {
      paeth = sse2::recon_paeth::<BYTES_PER_PIXEL>;
    }
    if BYTES_PER_PIXEL >= 4 && has_sse2 {
      sub = sse2::recon_sub::<BYTES_PER_PIXEL>;
      // This only affects i586 targets running with sse2, but we might as well
      // put it here.
      up = sse2::recon_up;
    }
  }
  //#[cfg(FALSE)]
  #[cfg(target_arch = "aarch64")]
  {
    let has_neon = std::arch::is_aarch64_feature_detected!("neon");
    if (BYTES_PER_PIXEL == 2 || BYTES_PER_PIXEL >= 4) && has_neon {
      // Note(Lokathor): I'm not sure why, but at ByPP==3 the scalar versions
      // actually work faster than the Neon versions even though Neon runs
      // better at ByPP==2. Might be something to do with register+op
      // scheduling, or something like that.
      paeth = neon::recon_paeth::<BYTES_PER_PIXEL>;
      sub = neon::recon_sub::<BYTES_PER_PIXEL>;
      average = neon::recon_average::<BYTES_PER_PIXEL>;
      average_top = neon::recon_average_top::<BYTES_PER_PIXEL>;
    }
    if has_neon {
      up = neon::recon_up;
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
      4 => unsafe { sub(line) },
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
