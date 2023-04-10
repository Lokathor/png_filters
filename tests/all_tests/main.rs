mod neon_tests;
mod sse2_tests;

#[test]
fn test_recon_sub_fallback() {
  let mut actual = [1, 2, 3, u8::MAX, 5, 6, 7, 8];
  png_filters::recon_sub_fallback::<1>(&mut actual);
  let expected = [1, 3, 6, 5, 10, 16, 23, 31];
  assert_eq!(expected, actual);
  //
  let mut actual = [1, 2, 3, u8::MAX, 5, 6, 7, 8];
  png_filters::recon_sub_fallback::<2>(&mut actual);
  let expected = [1, 2, 4, 1, 9, 7, 16, 15];
  assert_eq!(expected, actual);
  //
  let mut actual = [1, 2, 3, u8::MAX, 5, 6, 7, 8];
  png_filters::recon_sub_fallback::<4>(&mut actual);
  let expected = [1, 2, 3, u8::MAX, 6, 8, 10, 7];
  assert_eq!(expected, actual);
}

#[test]
fn test_recon_up_fallback() {
  let last_row = [12, 17, 127, 128, 255, 250, 7, 54];
  //
  let mut actual = [1, 2, 3, u8::MAX, 5, 6, 7, 8];
  png_filters::recon_up_fallback(&mut actual, &last_row);
  let expected = [13, 19, 130, 127, 4, 0, 14, 62];
  assert_eq!(expected, actual);
}

#[test]
fn test_recon_average_fallback() {
  let last_row = [12, 17, 127, 128, 255, 250, 7, 54];
  //
  let mut actual = [1, 2, 3, u8::MAX, 5, 6, 7, 8];
  png_filters::recon_average_fallback::<1>(&mut actual, &last_row);
  let expected = [7, 14, 73, 99, 182, 222, 121, 95];
  assert_eq!(expected, actual);
  //
  let mut actual = [1, 2, 3, u8::MAX, 5, 6, 7, 8];
  png_filters::recon_average_fallback::<2>(&mut actual, &last_row);
  let expected = [7, 10, 70, 68, 167, 165, 94, 117];
  assert_eq!(expected, actual);
  //
  let mut actual = [1, 2, 3, u8::MAX, 5, 6, 7, 8];
  png_filters::recon_average_fallback::<4>(&mut actual, &last_row);
  let expected = [7, 10, 66, 63, 136, 136, 43, 66];
  assert_eq!(expected, actual);
}

#[test]
fn test_recon_average_top_fallback() {
  let mut actual = [1, 2, 3, u8::MAX, 5, 6, 7, 8];
  png_filters::recon_average_top_fallback::<1>(&mut actual);
  let expected = [1, 2, 4, 1, 5, 8, 11, 13];
  assert_eq!(expected, actual);
  //
  let mut actual = [1, 2, 3, u8::MAX, 5, 6, 7, 8];
  png_filters::recon_average_top_fallback::<2>(&mut actual);
  let expected = [1, 2, 3, 0, 6, 6, 10, 11];
  assert_eq!(expected, actual);
  //
  let mut actual = [1, 2, 3, u8::MAX, 5, 6, 7, 8];
  png_filters::recon_average_top_fallback::<4>(&mut actual);
  let expected = [1, 2, 3, 255, 5, 7, 8, 135];
  assert_eq!(expected, actual);
}

#[test]
fn test_recon_paeth_fallback() {
  let last_row = [12, 17, 127, 128, 255, 250, 7, 54];
  //
  let mut actual = [1, 2, 3, u8::MAX, 5, 6, 7, 8];
  png_filters::recon_paeth_fallback::<1>(&mut actual, &last_row);
  let expected = [13, 19, 130, 129, 4, 10, 14, 62];
  assert_eq!(expected, actual);
  //
  let mut actual = [1, 2, 3, u8::MAX, 5, 6, 7, 8];
  png_filters::recon_paeth_fallback::<2>(&mut actual, &last_row);
  let expected = [13, 19, 130, 127, 4, 0, 11, 8];
  assert_eq!(expected, actual);
  //
  let mut actual = [1, 2, 3, u8::MAX, 5, 6, 7, 8];
  png_filters::recon_paeth_fallback::<4>(&mut actual, &last_row);
  let expected = [13, 19, 130, 127, 4, 0, 14, 62];
  assert_eq!(expected, actual);
}
