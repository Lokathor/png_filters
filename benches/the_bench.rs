#![feature(test)]
#![allow(clippy::identity_op)]

extern crate test;
use test::Bencher;

use png_filters::unfilter_lines;

fn rand_bytes<const BYTES_PER_PIXEL: usize>(width: usize, height: usize) -> Vec<u8> {
  let mut bytes = vec![0_u8; (1 + width * BYTES_PER_PIXEL) * height];
  getrandom::getrandom(&mut bytes).unwrap();
  bytes
}

#[bench]
fn bench_all_sub_1(b: &mut Bencher) {
  let mut bytes = rand_bytes::<1>(1024, 1024);
  bytes.chunks_exact_mut(1 + 1024).for_each(|chunk| {
    let (f, _line) = chunk.split_first_mut().unwrap();
    *f = 1;
  });
  //
  b.iter(|| unfilter_lines::<1>(bytes.chunks_exact_mut(1 + 1024 * 1)))
}

#[bench]
fn bench_all_sub_2(b: &mut Bencher) {
  let mut bytes = rand_bytes::<2>(1024, 1024);
  bytes.chunks_exact_mut(1 + 1024).for_each(|chunk| {
    let (f, _line) = chunk.split_first_mut().unwrap();
    *f = 1;
  });
  //
  b.iter(|| unfilter_lines::<2>(bytes.chunks_exact_mut(1 + 1024 * 2)))
}

#[bench]
fn bench_all_sub_3(b: &mut Bencher) {
  let mut bytes = rand_bytes::<3>(1024, 1024);
  bytes.chunks_exact_mut(1 + 1024).for_each(|chunk| {
    let (f, _line) = chunk.split_first_mut().unwrap();
    *f = 1;
  });
  //
  b.iter(|| unfilter_lines::<3>(bytes.chunks_exact_mut(1 + 1024 * 3)))
}

#[bench]
fn bench_all_sub_4(b: &mut Bencher) {
  let mut bytes = rand_bytes::<4>(1024, 1024);
  bytes.chunks_exact_mut(1 + 1024).for_each(|chunk| {
    let (f, _line) = chunk.split_first_mut().unwrap();
    *f = 1;
  });
  //
  b.iter(|| unfilter_lines::<4>(bytes.chunks_exact_mut(1 + 1024 * 4)))
}

#[bench]
fn bench_all_sub_6(b: &mut Bencher) {
  let mut bytes = rand_bytes::<6>(1024, 1024);
  bytes.chunks_exact_mut(1 + 1024).for_each(|chunk| {
    let (f, _line) = chunk.split_first_mut().unwrap();
    *f = 1;
  });
  //
  b.iter(|| unfilter_lines::<6>(bytes.chunks_exact_mut(1 + 1024 * 6)))
}

#[bench]
fn bench_all_sub_8(b: &mut Bencher) {
  let mut bytes = rand_bytes::<8>(1024, 1024);
  bytes.chunks_exact_mut(1 + 1024).for_each(|chunk| {
    let (f, _line) = chunk.split_first_mut().unwrap();
    *f = 1;
  });
  //
  b.iter(|| unfilter_lines::<8>(bytes.chunks_exact_mut(1 + 1024 * 8)))
}

// // //

#[bench]
fn bench_all_up(b: &mut Bencher) {
  let mut bytes = rand_bytes::<4>(1024, 1024);
  bytes.chunks_exact_mut(1 + 1024).for_each(|chunk| {
    let (f, _line) = chunk.split_first_mut().unwrap();
    *f = 2;
  });
  //
  b.iter(|| unfilter_lines::<4>(bytes.chunks_exact_mut(1 + 1024 * 4)))
}

// // //

#[bench]
fn bench_all_average_1(b: &mut Bencher) {
  let mut bytes = rand_bytes::<1>(1024, 1024);
  bytes.chunks_exact_mut(1 + 1024).for_each(|chunk| {
    let (f, _line) = chunk.split_first_mut().unwrap();
    *f = 3;
  });
  //
  b.iter(|| unfilter_lines::<1>(bytes.chunks_exact_mut(1 + 1024 * 1)))
}

#[bench]
fn bench_all_average_2(b: &mut Bencher) {
  let mut bytes = rand_bytes::<2>(1024, 1024);
  bytes.chunks_exact_mut(1 + 1024).for_each(|chunk| {
    let (f, _line) = chunk.split_first_mut().unwrap();
    *f = 3;
  });
  //
  b.iter(|| unfilter_lines::<2>(bytes.chunks_exact_mut(1 + 1024 * 2)))
}

#[bench]
fn bench_all_average_3(b: &mut Bencher) {
  let mut bytes = rand_bytes::<3>(1024, 1024);
  bytes.chunks_exact_mut(1 + 1024).for_each(|chunk| {
    let (f, _line) = chunk.split_first_mut().unwrap();
    *f = 3;
  });
  //
  b.iter(|| unfilter_lines::<3>(bytes.chunks_exact_mut(1 + 1024 * 3)))
}

#[bench]
fn bench_all_average_4(b: &mut Bencher) {
  let mut bytes = rand_bytes::<4>(1024, 1024);
  bytes.chunks_exact_mut(1 + 1024).for_each(|chunk| {
    let (f, _line) = chunk.split_first_mut().unwrap();
    *f = 3;
  });
  //
  b.iter(|| unfilter_lines::<4>(bytes.chunks_exact_mut(1 + 1024 * 4)))
}

#[bench]
fn bench_all_average_6(b: &mut Bencher) {
  let mut bytes = rand_bytes::<6>(1024, 1024);
  bytes.chunks_exact_mut(1 + 1024).for_each(|chunk| {
    let (f, _line) = chunk.split_first_mut().unwrap();
    *f = 3;
  });
  //
  b.iter(|| unfilter_lines::<6>(bytes.chunks_exact_mut(1 + 1024 * 6)))
}

#[bench]
fn bench_all_average_8(b: &mut Bencher) {
  let mut bytes = rand_bytes::<8>(1024, 1024);
  bytes.chunks_exact_mut(1 + 1024).for_each(|chunk| {
    let (f, _line) = chunk.split_first_mut().unwrap();
    *f = 3;
  });
  //
  b.iter(|| unfilter_lines::<8>(bytes.chunks_exact_mut(1 + 1024 * 8)))
}

// // //

#[bench]
fn bench_all_paeth_1(b: &mut Bencher) {
  let mut bytes = rand_bytes::<1>(1024, 1024);
  bytes.chunks_exact_mut(1 + 1024).for_each(|chunk| {
    let (f, _line) = chunk.split_first_mut().unwrap();
    *f = 4;
  });
  //
  b.iter(|| unfilter_lines::<1>(bytes.chunks_exact_mut(1 + 1024 * 1)))
}

#[bench]
fn bench_all_paeth_2(b: &mut Bencher) {
  let mut bytes = rand_bytes::<2>(1024, 1024);
  bytes.chunks_exact_mut(1 + 1024).for_each(|chunk| {
    let (f, _line) = chunk.split_first_mut().unwrap();
    *f = 4;
  });
  //
  b.iter(|| unfilter_lines::<2>(bytes.chunks_exact_mut(1 + 1024 * 2)))
}

#[bench]
fn bench_all_paeth_3(b: &mut Bencher) {
  let mut bytes = rand_bytes::<3>(1024, 1024);
  bytes.chunks_exact_mut(1 + 1024).for_each(|chunk| {
    let (f, _line) = chunk.split_first_mut().unwrap();
    *f = 4;
  });
  //
  b.iter(|| unfilter_lines::<3>(bytes.chunks_exact_mut(1 + 1024 * 3)))
}

#[bench]
fn bench_all_paeth_4(b: &mut Bencher) {
  let mut bytes = rand_bytes::<4>(1024, 1024);
  bytes.chunks_exact_mut(1 + 1024).for_each(|chunk| {
    let (f, _line) = chunk.split_first_mut().unwrap();
    *f = 4;
  });
  //
  b.iter(|| unfilter_lines::<4>(bytes.chunks_exact_mut(1 + 1024 * 4)))
}

#[bench]
fn bench_all_paeth_6(b: &mut Bencher) {
  let mut bytes = rand_bytes::<6>(1024, 1024);
  bytes.chunks_exact_mut(1 + 1024).for_each(|chunk| {
    let (f, _line) = chunk.split_first_mut().unwrap();
    *f = 4;
  });
  //
  b.iter(|| unfilter_lines::<6>(bytes.chunks_exact_mut(1 + 1024 * 6)))
}

#[bench]
fn bench_all_paeth_8(b: &mut Bencher) {
  let mut bytes = rand_bytes::<8>(1024, 1024);
  bytes.chunks_exact_mut(1 + 1024).for_each(|chunk| {
    let (f, _line) = chunk.split_first_mut().unwrap();
    *f = 4;
  });
  //
  b.iter(|| unfilter_lines::<8>(bytes.chunks_exact_mut(1 + 1024 * 8)))
}
