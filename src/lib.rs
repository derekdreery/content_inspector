extern crate memchr;

use memchr::memchr;
use std::cmp::min;
use std::fmt;

const MAX_SCAN_SIZE: usize = 1024;

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ContentType {
    BINARY,
    UTF_8,
    UTF_16LE,
    UTF_16BE,
    UTF_32LE,
    UTF_32BE,
}

impl ContentType {
    pub fn is_binary(self) -> bool {
        self == ContentType::BINARY
    }

    pub fn is_printable(self) -> bool {
        !self.is_binary()
    }
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ContentType::*;

        let name: &str = match *self {
            BINARY => "binary",
            UTF_8 => "UTF-8",
            UTF_16LE => "UTF-16LE",
            UTF_16BE => "UTF-16BE",
            UTF_32LE => "UTF-32LE",
            UTF_32BE => "UTF-32BE",
        };
        write!(f, "{}", name)
    }
}

/// Common byte order marks
/// (see https://en.wikipedia.org/wiki/Byte_order_mark)
static BYTE_ORDER_MARKS: &[(&[u8], ContentType)] = &[
    (&[0xEF, 0xBB, 0xBF], ContentType::UTF_8),
    // UTF-32 needs to be checked before UTF-16 (overlapping BOMs)
    (&[0x00, 0x00, 0xFE, 0xFF], ContentType::UTF_32BE),
    (&[0xFE, 0xFF, 0x00, 0x00], ContentType::UTF_32LE),
    (&[0xFE, 0xFF], ContentType::UTF_16BE),
    (&[0xFF, 0xFE], ContentType::UTF_16LE),
];

/// PDF header
static MAGIC_NUMBER_PDF: &[u8] = b"%PDF";

/// Try to guess (or determine) the type of content in the given buffer. If the buffer is
/// empty, the content type will be reported as UTF-8.
///
/// ```rust
/// use content_inspector::{ContentType, inspect};
///
/// assert_eq!(ContentType::UTF_8, inspect(b"Hello"));
/// assert_eq!(ContentType::BINARY, inspect(b"\xFF\xE0\x00\x10\x4A\x46\x49\x46\x00"));
/// ```
pub fn inspect(buffer: &[u8]) -> ContentType {
    use ContentType::*;

    // Scan the first few bytes for zero-bytes
    let scan_size = min(buffer.len(), MAX_SCAN_SIZE);
    let has_zero_bytes = memchr(0x00, &buffer[..scan_size]).is_some();

    if has_zero_bytes {
        for (bom, content_type) in BYTE_ORDER_MARKS {
            if buffer.starts_with(bom) {
                return *content_type;
            }
        }

        return BINARY;
    }

    if buffer.starts_with(MAGIC_NUMBER_PDF) {
        return BINARY;
    }

    UTF_8
}

#[cfg(test)]
mod tests {
    use {inspect, ContentType::*};

    #[test]
    fn test_empty_buffer_utf_8() {
        assert_eq!(UTF_8, inspect(b""));
    }

    #[test]
    fn test_text_utf8() {
        assert_eq!(UTF_8, inspect(include_bytes!("../testdata/text_UTF-8.txt")));
    }

    #[test]
    fn test_text_utf16le() {
        assert_eq!(
            UTF_16LE,
            inspect(include_bytes!("../testdata/text_UTF-16LE.txt"))
        );
    }

    #[test]
    fn test_text_utf32le() {
        assert_eq!(
            UTF_32LE,
            inspect(include_bytes!("../testdata/text_UTF-32LE.txt"))
        );
    }

    #[test]
    fn test_png() {
        assert_eq!(BINARY, inspect(include_bytes!("../testdata/test.png")));
    }

    #[test]
    fn test_jpg() {
        assert_eq!(BINARY, inspect(include_bytes!("../testdata/test.jpg")));
    }

    #[test]
    fn test_pdf() {
        assert_eq!(BINARY, inspect(include_bytes!("../testdata/test.pdf")));
    }
}