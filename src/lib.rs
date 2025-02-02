#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]
#![warn(clippy::cargo)]
#![allow(unknown_lints)]
#![warn(missing_copy_implementations)]
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(rust_2024_compatibility)]
#![warn(trivial_casts, trivial_numeric_casts)]
#![warn(unused_qualifications)]
#![warn(variant_size_differences)]
#![forbid(unsafe_code)]
// Enable feature callouts in generated documentation:
// https://doc.rust-lang.org/beta/unstable-book/language-features/doc-cfg.html
//
// This approach is borrowed from tokio.
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, feature(doc_alias))]

//! This crate provides an implementation of a Bubble Babble encoder and
//! decoder.
//!
//! The Bubble Babble encoding uses alternation of consonants and vowels to
//! encode binary data to pseudowords that can be pronounced more easily than
//! arbitrary lists of hexadecimal digits.
//!
//! Bubble Babble is part of the Digest libraries in [Perl][perl-bubblebabble]
//! and [Ruby][ruby-bubblebabble].
//!
//! [perl-bubblebabble]: https://metacpan.org/pod/Digest::BubbleBabble
//! [ruby-bubblebabble]: https://ruby-doc.org/stdlib-3.1.1/libdoc/digest/rdoc/Digest.html#method-c-bubblebabble
//!
//! # Usage
//!
//! You can encode binary data by calling [`encode`](encode()):
//!
//! ```
//! let encoded = boba::encode("Pineapple");
//! assert_eq!(encoded, "xigak-nyryk-humil-bosek-sonax");
//! ```
//!
//! Decoding binary data is done by calling [`decode`](decode()):
//!
//! ```
//! # use boba::DecodeError;
//! # fn example() -> Result<(), DecodeError> {
//! let decoded = boba::decode("xexax")?;
//! assert_eq!(decoded, vec![]);
//! # Ok(())
//! # }
//! # example().unwrap();
//! ```
//!
//! Decoding data is fallible and can return [`DecodeError`]. For example, all
//! Bubble Babble–encoded data has an ASCII alphabet, so attempting to decode an
//! emoji will fail.
//!
//! ```
//! # use boba::DecodeError;
//! let decoded = boba::decode("x🦀x");
//! // The `DecodeError` contains the offset of the first invalid byte.
//! assert_eq!(decoded, Err(DecodeError::InvalidByte(1)));
//! ```
//!
//! # Crate Features
//!
//! Boba is `no_std` compatible with a required dependency on the [`alloc`]
//! crate.

#![no_std]
#![doc(html_root_url = "https://docs.rs/boba/5.0.0")]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

mod decode;
mod encode;

/// Decoding errors from [`boba::decode`](decode()).
///
/// `decode` will return a `DecodeError` if:
///
/// - The input is not an ASCII string.
/// - The input contains an ASCII character outside of the Bubble Babble
///   encoding alphabet.
/// - The input does not start with a leading `x`.
/// - The input does not end with a trailing `x`.
/// - The decoded result does not checksum properly.
///
/// # Examples
///
/// ```
/// # use boba::DecodeError;
/// assert_eq!(boba::decode("x💎🦀x"), Err(DecodeError::InvalidByte(1)));
/// assert_eq!(boba::decode("x789x"), Err(DecodeError::InvalidByte(1)));
/// assert_eq!(boba::decode("yx"), Err(DecodeError::MalformedHeader));
/// assert_eq!(boba::decode("xy"), Err(DecodeError::MalformedTrailer));
/// assert_eq!(boba::decode(""), Err(DecodeError::Corrupted));
/// assert_eq!(boba::decode("z"), Err(DecodeError::Corrupted));
/// assert_eq!(boba::decode("xx"), Err(DecodeError::Corrupted));
/// ```
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum DecodeError {
    /// Checksum mismatch when decoding input.
    ChecksumMismatch,
    /// Corrupted input caused a decoding failure.
    Corrupted,
    /// Expected to process a consonant from the encoding alphabet, but got
    /// something else.
    ExpectedConsonant,
    /// Expected to process a vowel from the encoding alphabet, but got
    /// something else.
    ExpectedVowel,
    /// Input contained a byte not in the encoding alphabet at this position.
    InvalidByte(usize),
    /// Input was missing a leading `x` header.
    MalformedHeader,
    /// Input was missing a final `x` trailer.
    MalformedTrailer,
}

impl core::error::Error for DecodeError {}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ChecksumMismatch => f.write_str("Checksum mismatch"),
            Self::Corrupted => f.write_str("Corrupted input"),
            Self::ExpectedConsonant => f.write_str("Expected consonant, got something else"),
            Self::ExpectedVowel => f.write_str("Expected vowel, got something else"),
            Self::InvalidByte(pos) => write!(
                f,
                "Encountered byte outside of encoding alphabet at position {pos}"
            ),
            Self::MalformedHeader => f.write_str("Missing required 'x' header"),
            Self::MalformedTrailer => f.write_str("Missing required 'x' trailer"),
        }
    }
}

/// Encode a byte slice with the Bubble Babble encoding to a [`String`].
///
/// # Examples
///
/// ```
/// assert_eq!(boba::encode([]), "xexax");
/// assert_eq!(boba::encode("1234567890"), "xesef-disof-gytuf-katof-movif-baxux");
/// assert_eq!(boba::encode("Pineapple"), "xigak-nyryk-humil-bosek-sonax");
/// ```
#[must_use]
pub fn encode<T: AsRef<[u8]>>(data: T) -> String {
    encode::inner(data.as_ref())
}

/// Decode Bubble Babble-encoded byte slice to a [`Vec<u8>`](Vec).
///
/// # Examples
///
/// ```
/// # use boba::DecodeError;
/// # fn example() -> Result<(), DecodeError> {
/// assert_eq!(boba::decode("xexax")?, vec![]);
/// assert_eq!(boba::decode("xesef-disof-gytuf-katof-movif-baxux")?, b"1234567890");
/// assert_eq!(boba::decode("xigak-nyryk-humil-bosek-sonax")?, b"Pineapple");
/// # Ok(())
/// # }
/// # example().unwrap();
/// ```
///
/// # Errors
///
/// Decoding is fallible and might return [`DecodeError`] if:
///
/// - The input is not an ASCII string.
/// - The input contains an ASCII character outside of the Bubble Babble
///   encoding alphabet.
/// - The input does not start with a leading `x`.
/// - The input does not end with a trailing `x`.
/// - The decoded result does not checksum properly.
///
/// ```
/// # use boba::DecodeError;
/// assert_eq!(boba::decode("x💎🦀x"), Err(DecodeError::InvalidByte(1)));
/// assert_eq!(boba::decode("x789x"), Err(DecodeError::InvalidByte(1)));
/// assert_eq!(boba::decode("yx"), Err(DecodeError::MalformedHeader));
/// assert_eq!(boba::decode("xy"), Err(DecodeError::MalformedTrailer));
/// assert_eq!(boba::decode(""), Err(DecodeError::Corrupted));
/// assert_eq!(boba::decode("z"), Err(DecodeError::Corrupted));
/// assert_eq!(boba::decode("xx"), Err(DecodeError::Corrupted));
/// ```
pub fn decode<T: AsRef<[u8]>>(encoded: T) -> Result<Vec<u8>, DecodeError> {
    decode::inner(encoded.as_ref())
}

#[cfg(test)]
#[allow(clippy::non_ascii_literal)]
mod tests {
    use alloc::string::String;
    use alloc::vec;
    use core::fmt::Write as _;

    use crate::{decode, encode, DecodeError};

    #[test]
    fn encoder() {
        assert_eq!(encode([]), "xexax");
        assert_eq!(encode("1234567890"), "xesef-disof-gytuf-katof-movif-baxux");
        assert_eq!(encode("Pineapple"), "xigak-nyryk-humil-bosek-sonax");

        assert_eq!(
            encode("💎🦀❤️✨💪"),
            "xusan-zugom-vesin-zenom-bumun-tanav-zyvam-zomon-sapaz-bulin-dypux"
        );

        assert_eq!(encode("xyz!x6"), "xival-neved-cavuf-kexyx");
    }

    #[test]
    fn decoder() {
        assert_eq!(decode("xexax"), Ok(vec![]));
        assert_eq!(
            decode("xesef-disof-gytuf-katof-movif-baxux"),
            Ok(b"1234567890".to_vec())
        );
        assert_eq!(
            decode("xigak-nyryk-humil-bosek-sonax"),
            Ok(b"Pineapple".to_vec())
        );

        assert_eq!(
            decode("xusan-zugom-vesin-zenom-bumun-tanav-zyvam-zomon-sapaz-bulin-dypux"),
            Ok(String::from("💎🦀❤️✨💪").into_bytes())
        );

        assert_eq!(decode("xival-neved-cavuf-kexyx"), Ok(b"xyz!x6".to_vec()));
    }

    #[test]
    fn decode_error_sub_dash() {
        assert_eq!(
            decode("xesefxdisofxgytufxkatofxmovifxbaxux"),
            Err(DecodeError::ChecksumMismatch)
        );
    }

    #[test]
    fn decode_sub_vowel_to_consonant() {
        assert_eq!(
            decode("xssef-disof-gytuf-katof-movif-baxux"),
            Err(DecodeError::ExpectedVowel),
        );
    }

    #[test]
    fn decode_sub_consonant_to_vowel() {
        assert_eq!(
            decode("xeeef-disof-gytuf-katof-movif-baxux"),
            Err(DecodeError::ExpectedConsonant)
        );
    }

    #[test]
    fn decode_error() {
        assert_eq!(decode(""), Err(DecodeError::Corrupted));
        assert_eq!(decode("z"), Err(DecodeError::Corrupted));
        assert_eq!(decode("xy"), Err(DecodeError::MalformedTrailer));
        assert_eq!(decode("yx"), Err(DecodeError::MalformedHeader));
        assert_eq!(decode("xx"), Err(DecodeError::Corrupted));
        assert_eq!(decode("x💎🦀x"), Err(DecodeError::InvalidByte(1)));
        assert_eq!(decode("x789x"), Err(DecodeError::InvalidByte(1)));
    }

    #[test]
    fn decode_error_bad_alphabet() {
        assert_eq!(
            decode("xigak-nyryk-/umil-bosek-sonax"),
            Err(DecodeError::InvalidByte(12))
        );
        assert_eq!(decode(b"x\xFFx"), Err(DecodeError::InvalidByte(1)));
        assert_eq!(
            decode("xigak-nyryk-Humil-bosek-sonax"),
            Err(DecodeError::InvalidByte(12))
        );
        assert_eq!(
            decode("XIGAK-NYRYK-HUMIL-BOSEK-SONAX"),
            Err(DecodeError::Corrupted)
        );
        assert_eq!(
            decode("xIGAK-NYRYK-HUMIL-BOSEK-SONAX"),
            Err(DecodeError::MalformedTrailer)
        );
        assert_eq!(
            decode("xIGAK-NYRYK-HUMIL-BOSEK-SONAx"),
            Err(DecodeError::InvalidByte(1))
        );
    }

    #[test]
    fn error_display_is_not_empty() {
        let test_cases = [
            DecodeError::ChecksumMismatch,
            DecodeError::Corrupted,
            DecodeError::ExpectedConsonant,
            DecodeError::ExpectedVowel,
            DecodeError::InvalidByte(0),
            DecodeError::InvalidByte(123),
            DecodeError::MalformedHeader,
            DecodeError::MalformedTrailer,
        ];
        for tc in test_cases {
            let mut buf = String::new();
            write!(&mut buf, "{tc}").unwrap();
            assert!(!buf.is_empty());
        }
    }

    #[test]
    fn test_inner_triggers_decode_3_tuple_corrupted() {
        // This encoded input is designed to trigger a DecodeError::Corrupted.
        // It consists of a header and trailer ('x') framing the 6-byte chunk
        // "abab-b".  Within the chunk, the first three-tuple decodes to a value
        // where the computed 'high' component is 5 (>= 4), which violates the
        // valid range and causes an error.
        let encoded = b"xabab-bx";
        let result = decode(encoded);
        assert_eq!(result, Err(DecodeError::Corrupted));
    }
}

// Ensure code blocks in `README.md` compile.
//
// This module and macro declaration should be kept at the end of the file, in
// order to not interfere with code coverage.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod readme {}
