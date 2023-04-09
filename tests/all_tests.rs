use png_filters::*;

#[test]
fn test_recon_sub_fallback() {
  let mut expected = [1, 2, 3, u8::MAX, 5, 6, 7, 8];
  recon_sub_fallback::<1>(&mut expected);
  let actual = [1, 3, 6, 5, 10, 16, 23, 31];
  assert_eq!(expected, actual);
  //
  let mut expected = [1, 2, 3, u8::MAX, 5, 6, 7, 8];
  recon_sub_fallback::<2>(&mut expected);
  let actual = [1, 2, 4, 1, 9, 7, 16, 15];
  assert_eq!(expected, actual);
  //
  let mut expected = [1, 2, 3, u8::MAX, 5, 6, 7, 8];
  recon_sub_fallback::<4>(&mut expected);
  let actual = [1, 2, 3, u8::MAX, 6, 8, 10, 7];
  assert_eq!(expected, actual);
}
