#![cfg(any(target_arch = "x86", target_arch = "x86_64"))]

use std::arch::is_x86_feature_detected;

#[test]
fn test_recon_sub_sse2() {
  if is_x86_feature_detected!("sse2") {
    unsafe {
      let mut expected = [1, 2, 3, u8::MAX, 5, 6, 7, 8];
      png_filters::sse2::recon_sub::<1>(&mut expected);
      let actual = [1, 3, 6, 5, 10, 16, 23, 31];
      assert_eq!(expected, actual);
      //
      let mut expected = [1, 2, 3, u8::MAX, 5, 6, 7, 8];
      png_filters::sse2::recon_sub::<2>(&mut expected);
      let actual = [1, 2, 4, 1, 9, 7, 16, 15];
      assert_eq!(expected, actual);
      //
      let mut expected = [1, 2, 3, u8::MAX, 5, 6, 7, 8];
      png_filters::sse2::recon_sub::<4>(&mut expected);
      let actual = [1, 2, 3, u8::MAX, 6, 8, 10, 7];
      assert_eq!(expected, actual);
    }
  }
}

#[test]
fn test_recon_up_sse2() {
  if is_x86_feature_detected!("sse2") {
    unsafe {
      let last_row = [12, 17, 127, 128, 255, 250, 7, 54];
      //
      let mut expected = [1, 2, 3, u8::MAX, 5, 6, 7, 8];
      png_filters::sse2::recon_up(&mut expected, &last_row);
      let actual = [13, 19, 130, 127, 4, 0, 14, 62];
      assert_eq!(expected, actual);
    }
  }
}

#[test]
fn test_recon_average_sse2() {
  if is_x86_feature_detected!("sse2") {
    unsafe {
      let last_row = [12, 17, 127, 128, 255, 250, 7, 54];
      //
      let mut actual = [1, 2, 3, u8::MAX, 5, 6, 7, 8];
      png_filters::sse2::recon_average::<1>(&mut actual, &last_row);
      let expected = [7, 14, 73, 99, 182, 222, 121, 95];
      assert_eq!(expected, actual);
      //
      let mut actual = [1, 2, 3, u8::MAX, 5, 6, 7, 8];
      png_filters::sse2::recon_average::<2>(&mut actual, &last_row);
      let expected = [7, 10, 70, 68, 167, 165, 94, 117];
      assert_eq!(expected, actual);
      //
      let mut actual = [1, 2, 3, u8::MAX, 5, 6, 7, 8];
      png_filters::sse2::recon_average::<4>(&mut actual, &last_row);
      let expected = [7, 10, 66, 63, 136, 136, 43, 66];
      assert_eq!(expected, actual);
    }
  }
}

#[test]
fn test_recon_average_top_sse2() {
  if is_x86_feature_detected!("sse2") {
    unsafe {
      let mut actual = [1, 2, 3, u8::MAX, 5, 6, 7, 8];
      png_filters::sse2::recon_average_top::<1>(&mut actual);
      let expected = [1, 2, 4, 1, 5, 8, 11, 13];
      assert_eq!(expected, actual);
      //
      let mut actual = [1, 2, 3, u8::MAX, 5, 6, 7, 8];
      png_filters::sse2::recon_average_top::<2>(&mut actual);
      let expected = [1, 2, 3, 0, 6, 6, 10, 11];
      assert_eq!(expected, actual);
      //
      let mut actual = [1, 2, 3, u8::MAX, 5, 6, 7, 8];
      png_filters::sse2::recon_average_top::<4>(&mut actual);
      let expected = [1, 2, 3, 255, 5, 7, 8, 135];
      assert_eq!(expected, actual);
    }
  }
}

#[test]
fn test_recon_paeth_sse2() {
  if is_x86_feature_detected!("sse2") {
    unsafe {
      let last_row = [12, 17, 127, 128, 255, 250, 7, 54];
      //
      let mut actual = [1, 2, 3, u8::MAX, 5, 6, 7, 8];
      png_filters::sse2::recon_paeth::<1>(&mut actual, &last_row);
      let expected = [13, 19, 130, 129, 4, 10, 14, 62];
      assert_eq!(expected, actual);
      //
      let mut actual = [1, 2, 3, u8::MAX, 5, 6, 7, 8];
      png_filters::sse2::recon_paeth::<2>(&mut actual, &last_row);
      let expected = [13, 19, 130, 127, 4, 0, 11, 8];
      assert_eq!(expected, actual);
      //
      let mut actual = [1, 2, 3, u8::MAX, 5, 6, 7, 8];
      png_filters::sse2::recon_paeth::<4>(&mut actual, &last_row);
      let expected = [13, 19, 130, 127, 4, 0, 14, 62];
      assert_eq!(expected, actual);
    }
  }
}
