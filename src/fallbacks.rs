//! Functions in this module are always available, they don't depend on CPU
//! intrinsics or even on a specific CPU architecture.

/// `Recon(x) = Filt(x) + Recon(a)`
///
/// ## Panics
/// * `assert!(BYTES_PER_PIXEL <= 8);`
/// * `debug_assert_eq!(filtered_row.len() % BYTES_PER_PIXEL, 0);`
#[inline]
#[deny(unsafe_code)]
pub fn recon_sub<const BYTES_PER_PIXEL: usize>(filtered_row: &mut [u8]) {
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
pub fn recon_up(filtered_row: &mut [u8], previous_row: &[u8]) {
  debug_assert_eq!(filtered_row.len(), previous_row.len());
  //
  filtered_row.iter_mut().zip(previous_row.iter()).for_each(|(x, b)| *x = x.wrapping_add(*b))
}

/// `Recon(x) = Filt(x) + floor((Recon(a) + Recon(b)) / 2)`
///
/// ## Panic
/// * `assert!(BYTES_PER_PIXEL <= 8);`
/// * `debug_assert_eq!(filtered_row.len() % BYTES_PER_PIXEL, 0);`
/// * `debug_assert_eq!(filtered_row.len(), previous_row.len());`
#[inline]
#[deny(unsafe_code)]
pub fn recon_average<const BYTES_PER_PIXEL: usize>(filtered_row: &mut [u8], previous_row: &[u8]) {
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
pub fn recon_average_top<const BYTES_PER_PIXEL: usize>(filtered_row: &mut [u8]) {
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
pub fn recon_paeth<const BYTES_PER_PIXEL: usize>(filtered_row: &mut [u8], previous_row: &[u8]) {
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
