#![cfg(target_arch = "aarch64")]

use std::arch::is_aarch64_feature_detected;

#[test]
fn test_recon_sub_neon() {
  if is_aarch64_feature_detected!("neon") {
    unsafe {
      let mut expected = [1, 2, 3, u8::MAX, 5, 6, 7, 8];
      png_filters::neon::recon_sub::<1>(&mut expected);
      let actual = [1, 3, 6, 5, 10, 16, 23, 31];
      assert_eq!(expected, actual);
      //
      let mut expected = [1, 2, 3, u8::MAX, 5, 6, 7, 8];
      png_filters::neon::recon_sub::<2>(&mut expected);
      let actual = [1, 2, 4, 1, 9, 7, 16, 15];
      assert_eq!(expected, actual);
      //
      let mut expected = [1, 2, 3, u8::MAX, 5, 6, 7, 8];
      png_filters::neon::recon_sub::<4>(&mut expected);
      let actual = [1, 2, 3, u8::MAX, 6, 8, 10, 7];
      assert_eq!(expected, actual);
    }
  }
}

#[test]
fn test_recon_up_neon() {
  if is_aarch64_feature_detected!("neon") {
    unsafe {
      let last_row = [12, 17, 127, 128, 255, 250, 7, 54];
      //
      let mut expected = [1, 2, 3, u8::MAX, 5, 6, 7, 8];
      png_filters::neon::recon_up(&mut expected, &last_row);
      let actual = [13, 19, 130, 127, 4, 0, 14, 62];
      assert_eq!(expected, actual);
    }
  }
}

#[test]
#[cfg(FALSE)]
fn test_recon_average_neon() {
  if is_aarch64_feature_detected!("neon") {
    unsafe {
      let last_row = [12, 17, 127, 128, 255, 250, 7, 54];
      //
      let mut actual = [1, 2, 3, u8::MAX, 5, 6, 7, 8];
      png_filters::neon::recon_average::<1>(&mut actual, &last_row);
      let expected = [7, 14, 73, 99, 182, 222, 121, 95];
      assert_eq!(expected, actual);
      //
      let mut actual = [1, 2, 3, u8::MAX, 5, 6, 7, 8];
      png_filters::neon::recon_average::<2>(&mut actual, &last_row);
      let expected = [7, 10, 70, 68, 167, 165, 94, 117];
      assert_eq!(expected, actual);
      //
      let mut actual = [1, 2, 3, u8::MAX, 5, 6, 7, 8];
      png_filters::neon::recon_average::<4>(&mut actual, &last_row);
      let expected = [7, 10, 66, 63, 136, 136, 43, 66];
      assert_eq!(expected, actual);
    }
  }
}
