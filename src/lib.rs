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

pub mod fallbacks;
#[cfg(target_arch = "aarch64")]
pub mod neon;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub mod sse2;
