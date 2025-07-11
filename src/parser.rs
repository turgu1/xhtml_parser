//! Parser for XML/XHTML documents.
//!
//! This module provides functionality to parse XML content, handling various node types and attributes.
//! It defines the `parser()` method for processing XML data.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::inline_always)]

use crate::defs::{NodeIdx, ParseXmlError, XmlIdx, XmlLocation};
use crate::document::Document;
use crate::node_type::NodeType;

use kmp::kmp_find;
use phf::phf_map;

//use memchr::memchr2;
use memchr::{memchr, memchr2};

use core::ops::Range;

type XmlRange = Range<XmlIdx>;

enum State {
    Start,
    ReadStartOfTag,
    ReadTagOpen,
    ReadTagClose,
    ReadAttribute,
    ReadPCData,
    End,
}

const LESS_THAN: u8 = b'<';
const GREATER_THAN: u8 = b'>';
const SLASH: u8 = b'/';
const EQUAL: u8 = b'=';
const EXCLAMATION_MARK: u8 = b'!';
const QUESTION_MARK: u8 = b'?';
const AMPERSAND: u8 = b'&';
const SEMI_COLON: u8 = b';';
const HASH: u8 = b'#';
const X_CHAR: u8 = b'x';
const LEFT_BRACKET: u8 = b'[';
const RIGHT_BRACKET: u8 = b']';
const SPACE: u8 = b' ';

#[cfg(feature = "namespace_removal")]
const COLON: u8 = b':';

#[cfg(not(feature = "parse_escapes"))]
const NEWLINE: u8 = b'\n';

#[cfg(not(feature = "parse_escapes"))]
const CARRIAGE_RETURN: u8 = b'\r';

#[allow(dead_code)]
#[derive(Clone, Copy)]

#[rustfmt::skip]
pub(crate) enum Chartype {
    ParsePCData   =   1,  // &, \r
    ParseAtrNorm  =   4,  // &, \r, \n, space, tab
    Space         =   8,  // \r, \n, space, tab
    ParseCloseTag =  16,  // \r, \n, space, tab, >
    Symbol        =  64,  // Any symbol > 127, a-z, A-Z, 0-9, _, :, -, .
    StartSymBol   = 128,  // Any symbol > 127, a-z, A-Z, _, :
}

#[rustfmt::skip]
pub(crate) const CHARTYPE_TABLE: [u8; 256] = [
      0,   0,   0,   0,   0,   0,   0,   0,   0,  28,  28,   0,   0,  29,   0,   0, // 0-15
      0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0, // 16-31
     28,   0,   0,   0,   0,   0,   5,   0,   0,   0,   0,   0,   0,  64,  64,   0, // 32-47
     64,  64,  64,  64,  64,  64,  64,  64,  64,  64, 192,   0,   0,   0,  16,   0, // 48-63
      0, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, // 64-79
    192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192,   0,   0,   0,   0, 192, // 80-95
      0, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, // 96-111
    192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192,   0,   0,   0,   0,   0, // 112-127
    192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, // 128+
    192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192,
    192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192,
    192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192,
    192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192,
    192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192,
    192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192,
    192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192,
];

#[rustfmt::skip]
static ENTITIES_MAP: phf::Map<&'static [u8], &'static [u8]> = phf_map! {
    b"nbsp"   => b"\xC2\xA0",     // no-break space = non-breaking space, U+00A0 ISOnum
    b"iexcl"  => b"\xC2\xA1",     // inverted exclamation mark, U+00A1 ISOnum
    b"cent"   => b"\xC2\xA2",     // cent sign, U+00A2 ISOnum
    b"pound"  => b"\xC2\xA3",     // pound sign, U+00A3 ISOnum
    b"curren" => b"\xC2\xA4",     // currency sign, U+00A4 ISOnum
    b"yen"    => b"\xC2\xA5",     // yen sign = yuan sign, U+00A5 ISOnum
    b"brvbar" => b"\xC2\xA6",     // broken bar = broken vertical bar, U+00A6 ISOnum
    b"sect"   => b"\xC2\xA7",     // section sign, U+00A7 ISOnum
    b"uml"    => b"\xC2\xA8",     // diaeresis = spacing diaeresis, U+00A8 ISOdia
    b"copy"   => b"\xC2\xA9",     // copyright sign, U+00A9 ISOnum
    b"ordf"   => b"\xC2\xAA",     // feminine ordinal indicator, U+00AA ISOnum
    b"laquo"  => b"\xC2\xAB",     // left-pointing double angle quotation mark = left pointing guillemet, U+00AB ISOnum
    b"not"    => b"\xC2\xAC",     // not sign = angled dash, U+00AC ISOnum
    b"shy"    => b"\xC2\xAD",     // soft hyphen = discretionary hyphen, U+00AD ISOnum
    b"reg"    => b"\xC2\xAE",     // registered sign = registered trade mark sign, U+00AE ISOnum
    b"macr"   => b"\xC2\xAF",     // macron = spacing macron = overline = APL overbar, U+00AF ISOdia
    b"deg"    => b"\xC2\xB0",     // degree sign, U+00B0 ISOnum
    b"plusmn" => b"\xC2\xB1",     // plus-minus sign = plus-or-minus sign, U+00B1 ISOnum
    b"sup2"   => b"\xC2\xB2",     // superscript two = superscript digit two = squared, U+00B2 ISOnum
    b"sup3"   => b"\xC2\xB3",     // superscript three = superscript digit three = cubed, U+00B3 ISOnum
    b"acute"  => b"\xC2\xB4",     // acute accent = spacing acute, U+00B4 ISOdia
    b"micro"  => b"\xC2\xB5",     // micro sign, U+00B5 ISOnum
    b"para"   => b"\xC2\xB6",     // pilcrow sign = paragraph sign, U+00B6 ISOnum
    b"middot" => b"\xC2\xB7",     // middle dot = Georgian comma = Greek middle dot, U+00B7 ISOnum
    b"cedil"  => b"\xC2\xB8",     // cedilla = spacing cedilla, U+00B8 ISOdia
    b"sup1"   => b"\xC2\xB9",     // superscript one = superscript digit one, U+00B9 ISOnum
    b"ordm"   => b"\xC2\xBA",     // masculine ordinal indicator, U+00BA ISOnum
    b"raquo"  => b"\xC2\xBB",     // right-pointing double angle quotation mark = right pointing guillemet, U+00BB ISOnum
    b"frac14" => b"\xC2\xBC",     // vulgar fraction one quarter = fraction one quarter, U+00BC ISOnum
    b"frac12" => b"\xC2\xBD",     // vulgar fraction one half = fraction one half, U+00BD ISOnum
    b"frac34" => b"\xC2\xBE",     // vulgar fraction three quarters = fraction three quarters, U+00BE ISOnum
    b"iquest" => b"\xC2\xBF",     // inverted question mark = turned question mark, U+00BF ISOnum
    b"Agrave" => b"\xC3\x80",     // latin capital letter A with grave = latin capital letter A grave, U+00C0 ISOlat1
    b"Aacute" => b"\xC3\x81",     // latin capital letter A with acute, U+00C1 ISOlat1
    b"Acirc"  => b"\xC3\x82",     // latin capital letter A with circumflex, U+00C2 ISOlat1
    b"Atilde" => b"\xC3\x83",     // latin capital letter A with tilde, U+00C3 ISOlat1
    b"Auml"   => b"\xC3\x84",     // latin capital letter A with diaeresis, U+00C4 ISOlat1
    b"Aring"  => b"\xC3\x85",     // latin capital letter A with ring above = latin capital letter A ring, U+00C5 ISOlat1
    b"AElig"  => b"\xC3\x86",     // latin capital letter AE = latin capital ligature AE, U+00C6 ISOlat1
    b"Ccedil" => b"\xC3\x87",     // latin capital letter C with cedilla, U+00C7 ISOlat1
    b"Egrave" => b"\xC3\x88",     // latin capital letter E with grave, U+00C8 ISOlat1
    b"Eacute" => b"\xC3\x89",     // latin capital letter E with acute, U+00C9 ISOlat1
    b"Ecirc"  => b"\xC3\x8A",     // latin capital letter E with circumflex, U+00CA ISOlat1
    b"Euml"   => b"\xC3\x8B",     // latin capital letter E with diaeresis, U+00CB ISOlat1
    b"Igrave" => b"\xC3\x8C",     // latin capital letter I with grave, U+00CC ISOlat1
    b"Iacute" => b"\xC3\x8D",     // latin capital letter I with acute, U+00CD ISOlat1
    b"Icirc"  => b"\xC3\x8E",     // latin capital letter I with circumflex, U+00CE ISOlat1
    b"Iuml"   => b"\xC3\x8F",     // latin capital letter I with diaeresis, U+00CF ISOlat1
    b"ETH"    => b"\xC3\x90",     // latin capital letter ETH, U+00D0 ISOlat1
    b"Ntilde" => b"\xC3\x91",     // latin capital letter N with tilde, U+00D1 ISOlat1
    b"Ograve" => b"\xC3\x92",     // latin capital letter O with grave, U+00D2 ISOlat1
    b"Oacute" => b"\xC3\x93",     // latin capital letter O with acute, U+00D3 ISOlat1
    b"Ocirc"  => b"\xC3\x94",     // latin capital letter O with circumflex, U+00D4 ISOlat1
    b"Otilde" => b"\xC3\x95",     // latin capital letter O with tilde, U+00D5 ISOlat1
    b"Ouml"   => b"\xC3\x96",     // latin capital letter O with diaeresis, U+00D6 ISOlat1
    b"times"  => b"\xC3\x97",     // multiplication sign, U+00D7 ISOnum
    b"Oslash" => b"\xC3\x98",     // latin capital letter O with stroke = latin capital letter O slash, U+00D8 ISOlat1
    b"Ugrave" => b"\xC3\x99",     // latin capital letter U with grave, U+00D9 ISOlat1
    b"Uacute" => b"\xC3\x9A",     // latin capital letter U with acute, U+00DA ISOlat1
    b"Ucirc"  => b"\xC3\x9B",     // latin capital letter U with circumflex, U+00DB ISOlat1
    b"Uuml"   => b"\xC3\x9C",     // latin capital letter U with diaeresis, U+00DC ISOlat1
    b"Yacute" => b"\xC3\x9D",     // latin capital letter Y with acute, U+00DD ISOlat1
    b"THORN"  => b"\xC3\x9E",     // latin capital letter THORN, U+00DE ISOlat1
    b"szlig"  => b"\xC3\x9F",     // latin small letter sharp s = ess-zed, U+00DF ISOlat1
    b"agrave" => b"\xC3\xA0",     // latin small letter a with grave = latin small letter a grave, U+00E0 ISOlat1
    b"aacute" => b"\xC3\xA1",     // latin small letter a with acute, U+00E1 ISOlat1
    b"acirc"  => b"\xC3\xA2",     // latin small letter a with circumflex, U+00E2 ISOlat1
    b"atilde" => b"\xC3\xA3",     // latin small letter a with tilde, U+00E3 ISOlat1
    b"auml"   => b"\xC3\xA4",     // latin small letter a with diaeresis, U+00E4 ISOlat1
    b"aring"  => b"\xC3\xA5",     // latin small letter a with ring above = latin small letter a ring, U+00E5 ISOlat1
    b"aelig"  => b"\xC3\xA6",     // latin small letter ae = latin small ligature ae, U+00E6 ISOlat1
    b"ccedil" => b"\xC3\xA7",     // latin small letter c with cedilla, U+00E7 ISOlat1
    b"egrave" => b"\xC3\xA8",     // latin small letter e with grave, U+00E8 ISOlat1
    b"eacute" => b"\xC3\xA9",     // latin small letter e with acute, U+00E9 ISOlat1
    b"ecirc"  => b"\xC3\xAA",     // latin small letter e with circumflex, U+00EA ISOlat1
    b"euml"   => b"\xC3\xAB",     // latin small letter e with diaeresis, U+00EB ISOlat1
    b"igrave" => b"\xC3\xAC",     // latin small letter i with grave, U+00EC ISOlat1
    b"iacute" => b"\xC3\xAD",     // latin small letter i with acute, U+00ED ISOlat1
    b"icirc"  => b"\xC3\xAE",     // latin small letter i with circumflex, U+00EE ISOlat1
    b"iuml"   => b"\xC3\xAF",     // latin small letter i with diaeresis, U+00EF ISOlat1
    b"eth"    => b"\xC3\xB0",     // latin small letter eth, U+00F0 ISOlat1
    b"ntilde" => b"\xC3\xB1",     // latin small letter n with tilde, U+00F1 ISOlat1
    b"ograve" => b"\xC3\xB2",     // latin small letter o with grave, U+00F2 ISOlat1
    b"oacute" => b"\xC3\xB3",     // latin small letter o with acute, U+00F3 ISOlat1
    b"ocirc"  => b"\xC3\xB4",     // latin small letter o with circumflex, U+00F4 ISOlat1
    b"otilde" => b"\xC3\xB5",     // latin small letter o with tilde, U+00F5 ISOlat1
    b"ouml"   => b"\xC3\xB6",     // latin small letter o with diaeresis, U+00F6 ISOlat1
    b"divide" => b"\xC3\xB7",     // division sign, U+00F7 ISOnum
    b"oslash" => b"\xC3\xB8",     // latin small letter o with stroke, = latin small letter o slash, U+00F8 ISOlat1
    b"ugrave" => b"\xC3\xB9",     // latin small letter u with grave, U+00F9 ISOlat1
    b"uacute" => b"\xC3\xBA",     // latin small letter u with acute, U+00FA ISOlat1
    b"ucirc"  => b"\xC3\xBB",     // latin small letter u with circumflex, U+00FB ISOlat1
    b"uuml"   => b"\xC3\xBC",     // latin small letter u with diaeresis, U+00FC ISOlat1
    b"yacute" => b"\xC3\xBD",     // latin small letter y with acute, U+00FD ISOlat1
    b"thorn"  => b"\xC3\xBE",     // latin small letter thorn, U+00FE ISOlat1
    b"yuml"   => b"\xC3\xBF",     // latin small letter y with diaeresis, U+00FF ISOlat1
    b"quot"   => b"\"",           // quotation mark, U+0022 ISOnum
    b"amp"    => b"&",            // ampersand, U+0026 ISOnum
    b"lt"     => b"<",            // less-than sign, U+003C ISOnum
    b"gt"     => b">",            // greater-than sign, U+003E ISOnum
    b"apos"   => b"'",            // apostrophe = APL quote, U+0027 ISOnum
    b"OElig"  => b"\xC5\x92",     // latin capital ligature OE, U+0152 ISOlat2
    b"oelig"  => b"\xC5\x93",     // latin small ligature oe, U+0153 ISOlat2
    b"Scaron" => b"\xC5\xA0",     // latin capital letter S with caron, U+0160 ISOlat2
    b"scaron" => b"\xC5\xA1",     // latin small letter s with caron, U+0161 ISOlat2
    b"Yuml"   => b"\xC5\xB8",     // latin capital letter Y with diaeresis, U+0178 ISOlat2
    b"circ"   => b"\xCB\x86",     // modifier letter circumflex accent, U+02C6 ISOpub
    b"tilde"  => b"\xCB\x9C",     // small tilde, U+02DC ISOdia
    b"ensp"   => b"\xE2\x80\x82", // en space, U+2002 ISOpub
    b"emsp"   => b"\xE2\x80\x83", // em space, U+2003 ISOpub
    b"thinsp" => b"\xE2\x80\x89", // thin space, U+2009 ISOpub
    b"zwnj"   => b"\xE2\x80\x8C", // zero width non-joiner, U+200C NEW RFC 2070
    b"zwj"    => b"\xE2\x80\x8D", // zero width joiner, U+200D NEW RFC 2070
    b"lrm"    => b"\xE2\x80\x8E", // left-to-right mark, U+200E NEW RFC 2070
    b"rlm"    => b"\xE2\x80\x8F", // right-to-left mark, U+200F NEW RFC 2070
    b"ndash"  => b"\xE2\x80\x93", // en dash, U+2013 ISOpub
    b"mdash"  => b"\xE2\x80\x94", // em dash, U+2014 ISOpub
    b"lsquo"  => b"\xE2\x80\x98", // left single quotation mark, U+2018 ISOnum
    b"rsquo"  => b"\xE2\x80\x99", // right single quotation mark, U+2019 ISOnum
    b"sbquo"  => b"\xE2\x80\x9A", // single low-9 quotation mark, U+201A NEW
    b"ldquo"  => b"\xE2\x80\x9C", // left double quotation mark,  U+201C ISOnum
    b"rdquo"  => b"\xE2\x80\x9D", // right double quotation mark, U+201D ISOnum
    b"bdquo"  => b"\xE2\x80\x9E", // double low-9 quotation mark, U+201E NEW
    b"dagger" => b"\xE2\x80\xA0", // dagger, U+2020 ISOpub
    b"Dagger" => b"\xE2\x80\xA1", // double dagger, U+2021 ISOpub
    b"permil" => b"\xE2\x80\xB0", // per mille sign, U+2030 ISOtech
    b"lsaquo" => b"\xE2\x80\xB9", // single left-pointing angle quotation mark, U+2039 ISO proposed
    b"rsaquo" => b"\xE2\x80\xBA", // single right-pointing angle quotation mark, U+203A ISO proposed
    b"euro"   => b"\xE2\x80\xAC", // euro sign, U+20AC NEW
};

macro_rules! search_char {
    ($needle:expr, $haystack:expr) => {
        memchr($needle, $haystack)
        //$haystack.iter().position(|&c| c == $needle)
    };
}

struct Parent {
    parent_idx: NodeIdx,
    last_child_idx: NodeIdx,
}

impl Parent {
    fn new(parent_idx: NodeIdx) -> Self {
        Self {
            parent_idx,
            last_child_idx: 0,
        }
    }
}

impl Document {
    /// Skips a specific slice in the XML buffer, returning the next position after the slice.
    ///
    /// This method scans the XML buffer starting from the current pointer position `p`
    /// and skips over the specified slice.
    /// If the slice is not found, it returns `None`.
    ///
    /// # Arguments
    /// * `p` - The current position in the XML buffer
    /// * `max` - The maximum number of bytes to search for the slice
    /// * `slice` - The slice to skip, represented as a byte slice
    ///
    /// # Returns
    /// `Option<XmlIdx>` - The updated pointer position after skipping the slice,
    /// or `None` if the end of the XML buffer is reached or the slice is not found.
    #[inline(always)]
    fn skip_after_slice(&self, p: XmlIdx, max: XmlIdx, slice: &[u8]) -> Option<XmlIdx> {
        if p >= self.xml.len() as XmlIdx {
            None
        } else {
            let max_pos: XmlIdx = (p + max).min(self.xml.len() as XmlIdx);
            kmp_find(slice, &self.xml[p as usize..max_pos as usize])
                .map(|pos| p + pos as XmlIdx + slice.len() as XmlIdx)
        }
    }

    /// Skips characters of a specific type in the XML buffer, returning the next position that does not match the chartype.
    ///
    /// This method scans the XML buffer starting from the current pointer position
    /// and skips over characters that match the specified chartype.
    /// If the end of the buffer is reached, `None` is returned.
    ///
    /// # Arguments
    /// * `p` - The current position in the XML buffer
    /// * `chartype` - The chartype to skip, represented as a bitmask
    ///
    /// # Returns
    /// `Some(XmlIdx)` - The updated pointer position after skipping characters of the specified chartype,
    /// or `None` if the end of the XML buffer is reached.
    #[inline(always)]
    fn skip_chartype(&self, p: XmlIdx, chartype: Chartype) -> Option<XmlIdx> {
        if p >= self.xml.len() as XmlIdx {
            None
        } else {
            (self.xml[p as usize..])
                .iter()
                .position(|&byte| !Self::is_of_type(byte, chartype))
                .map(|pos| p + pos as XmlIdx)
        }
    }

    /// Scans a range in the XML buffer for a specific character type and returns the position of the first occurrence.
    ///
    /// This method searches for the first character in the specified range that matches the given chartype.
    /// If a character matching the chartype is found, it returns the position of that character.
    /// If no such character is found, it returns `None`.
    ///
    /// # Arguments
    /// * `range` - The range within the XML buffer to search.
    /// * `chartype` - The chartype to search for, represented as a `Chartype` enum.
    ///
    /// # Returns
    /// `Option<XmlIdx>` - The position of the first occurrence of the character matching the chartype,
    /// or `None` if no such character is found within the specified range.
    #[inline(always)]
    fn scan_range_for_chartype(&self, range: XmlRange, chartype: Chartype) -> Option<XmlIdx> {
        (self.xml[range.start as usize..range.end as usize])
            .iter()
            .position(|&byte| Self::is_of_type(byte, chartype))
            .map(|pos| range.start + pos as XmlIdx)
    }

    #[cfg(not(feature = "parse_escapes"))]
    /// Scans a range in the XML buffer for a specific character and returns the position of the first occurrence.
    ///
    /// This method searches for the first occurrence of the specified character within the given range.
    /// If the character is found, it returns the position of that character.
    /// If the character is not found, it returns `None`.
    ///
    /// # Arguments
    /// * `range` - The range within the XML buffer to search
    /// * `target_char` - The byte value of the character to search for
    ///
    /// # Returns
    /// `Option<XmlIdx>` - The position of the first occurrence of the character,
    /// or `None` if the character is not found within the specified range.
    #[inline(always)]
    fn scan_range_for_char(&self, range: XmlRange, target_char: u8) -> Option<XmlIdx> {
        search_char!(
            target_char,
            &self.xml[range.start as usize..range.end as usize]
        )
        .map(|pos| range.start + pos as XmlIdx)
    }

    /// Scans the XML buffer until a specific character is found.
    ///
    /// This method searches for the specified character starting from the current position `p`
    /// and returns the position of the found character. If the character is not found,
    /// it returns `None`.
    ///
    /// # Arguments
    /// * `p` - The current position in the XML buffer
    /// * `target_char` - The byte value of the character to search for
    ///
    /// # Returns
    /// `Option<XmlIdx>` - The position of the found character, or `None` if the character is not found
    /// or if the position `p` is beyond the end
    /// of the XML buffer.
    #[inline(always)]
    fn scan_until_char(&self, p: XmlIdx, target_char: u8) -> Option<XmlIdx> {
        if p >= self.xml.len() as XmlIdx {
            None
        } else {
            search_char!(target_char, &self.xml[p as usize..]).map(|pos| p + pos as XmlIdx)
        }
    }

    /// Scans the XML buffer until a character of a specific type is found.
    ///
    /// This method searches for the first character that matches the specified chartype
    /// starting from the current position `p` and returns the position of the found character.
    /// If no character matching the chartype is found, it returns `None`.
    ///
    /// # Arguments
    /// * `p` - The current position in the XML buffer
    /// * `chartype` - The chartype to search for, represented as a bitmask
    ///
    /// # Returns
    /// `Option<XmlIdx>` - The position of the found character matching the chartype,
    /// or `None` if no such character is found within the specified range or if the position `p`
    /// is beyond the end of the XML buffer.
    #[inline(always)]
    fn scan_until_chartype(&self, p: XmlIdx, chartype: Chartype) -> Option<XmlIdx> {
        if p >= self.xml.len() as XmlIdx {
            None
        } else {
            (self.xml[p as usize..])
                .iter()
                .position(|&byte| Self::is_of_type(byte, chartype))
                .map(|pos| p + pos as XmlIdx)
        }
    }

    /// Scans the XML buffer until one of two specific characters is found.
    ///
    /// This method searches for either `char1` or `char2` starting from the current position `p`
    /// and returns the position of the found character. If neither character is found,
    /// it returns `None`.
    ///
    /// # Arguments
    /// * `p` - The current position in the XML buffer
    /// * `char1` - The first byte value of the character to search for
    /// * `char2` - The second byte value of the character to search for
    ///
    /// # Returns
    /// `Option<XmlIdx>` - The position of the found character, or `None` if neither character is found
    /// or if the position `p` is beyond the end
    /// of the XML buffer.
    #[inline(always)]
    fn scan_until_one_of_2_chars(&self, p: XmlIdx, char1: u8, char2: u8) -> Option<XmlIdx> {
        if p >= self.xml.len() as XmlIdx {
            None
        } else {
            memchr2(char1, char2, &self.xml[p as usize..]).map(|pos| p + pos as XmlIdx)
        }
    }

    /// Displays XML content around an error position for debugging purposes.
    ///
    /// This method extracts a 60-character window (30 characters before and after)
    /// around the error position to provide context when reporting parsing errors.
    /// The error position is marked with `[*]` in the output.
    ///
    /// # Arguments
    /// * `pos` - The byte position in the XML where the error occurred
    ///
    /// # Returns
    /// A formatted string showing the XML context around the error position    
    #[inline]
    fn show_xml_around_error(&self, pos: XmlIdx) -> String {
        let start = pos.saturating_sub(30);

        let end = if (pos + 30) < self.xml.len() as XmlIdx {
            pos + 30
        } else {
            self.xml.len() as XmlIdx
        };

        format!(
            "...{}[*]{}...",
            std::str::from_utf8(&self.xml[start as usize..pos as usize])
                .unwrap_or("non valid utf-8"),
            std::str::from_utf8(&self.xml[pos as usize..end as usize]).unwrap_or("non valid utf-8")
        )
    }

    /// Creates a standardized parsing error with context information.
    ///
    /// This helper method generates a `ParseXmlError::InvalidXml` with a descriptive
    /// message that includes the error position and surrounding XML content for
    /// better debugging.
    ///
    /// # Arguments
    /// * `msg` - The error message describing what went wrong
    /// * `pos` - The byte position in the XML where the error occurred
    ///
    /// # Returns
    /// A `Result` containing the formatted parsing error
    #[inline]
    fn invalid(&self, msg: &str, pos: XmlIdx) -> Result<(), ParseXmlError> {
        Err(ParseXmlError::InvalidXml(format!(
            "{}. at position {}: {}",
            msg,
            pos,
            self.show_xml_around_error(pos)
        )))
    }

    /// Validates that a closing tag matches its corresponding opening tag.
    ///
    /// This method ensures XML well-formedness by verifying that closing tags
    /// have the same name as their matching opening tags. It extracts the tag
    /// names from their respective ranges and compares them.
    ///
    /// # Arguments
    /// * `parent_idx` - The node index of the parent element (opening tag)
    /// * `range` - The byte range containing the closing tag name
    ///
    /// # Returns
    /// `Ok(())` if the tags match, or a parsing error if they don't match
    /// or if the parent node is not an element
    #[inline(always)]
    fn check_closing_tag(
        &self,
        parent_idx: NodeIdx,
        location: XmlLocation,
    ) -> Result<(), ParseXmlError> {
        if let NodeType::Element { name, .. } = self.nodes[parent_idx as usize].node_type() {
            #[cfg(not(feature = "use_cstr"))]
            {
                let tag_name = self.get_str_from_location(name.clone());
                let closing_tag = self.get_str_from_location(location.clone());
                if tag_name != closing_tag {
                    return self.invalid(
                        &format!(
                            "Closing tag '{closing_tag}' does not match opening tag '{tag_name}'"
                        ),
                        location.start,
                    );
                }
            }
            #[cfg(feature = "use_cstr")]
            {
                let tag_name = std::ffi::CStr::from_bytes_until_nul(&self.xml[*name as usize..])
                    .or_else(|_| Err(ParseXmlError::InternalError))?;
                let closing_tag =
                    std::ffi::CStr::from_bytes_until_nul(&self.xml[location as usize..])
                        .or_else(|_| Err(ParseXmlError::InternalError))?;

                if tag_name != closing_tag {
                    return self.invalid(
                        &format!(
                            "Closing tag '{}' does not match opening tag '{}'",
                            self.get_str_from_location(location),
                            self.get_str_from_location(*name)
                        ),
                        location,
                    );
                }
            }
        } else {
            #[cfg(feature = "use_cstr")]
            let position = location;

            #[cfg(not(feature = "use_cstr"))]
            let position = location.start;

            return self.invalid("Expected an element node for closing tag", position);
        }
        Ok(())
    }

    /// Converts a byte slice containing decimal digits to a u32.
    ///
    /// This method processes the byte slice, ignoring any non-digit characters,
    /// and accumulates the value of the decimal digits into a u32.
    ///
    /// # Arguments
    /// * `s` - A byte slice potentially containing decimal digits
    /// # Returns
    /// A u32 representing the decimal value of the digits found in the input
    #[inline(always)]
    fn decimal(s: &[u8]) -> u32 {
        s.iter().fold(0u32, |acc, &c| {
            if c.is_ascii_digit() {
                acc * 10 + u32::from(c - b'0')
            } else {
                acc // Ignore non-digit characters
            }
        })
    }

    /// Converts a hexadecimal character to its u32 value.
    ///
    /// This method handles both uppercase and lowercase hexadecimal digits,
    /// converting them to their corresponding u32 values. It also supports
    /// digits '0'-'9' and letters 'A'-'F' or 'a'-'f'.
    ///
    /// # Arguments
    /// * `c` - A byte representing a hexadecimal character
    ///
    /// # Returns
    /// A u32 representing the value of the hexadecimal character
    #[inline(always)]
    fn hex_val(c: u8) -> u32 {
        if c.is_ascii_uppercase() {
            u32::from(c - b'A' + 10)
        } else if c.is_ascii_lowercase() {
            u32::from(c - b'a' + 10)
        } else {
            u32::from(c - b'0')
        }
    }

    /// Converts a byte slice containing hexadecimal digits to a u32.
    ///
    /// This method processes the byte slice, ignoring any non-hexadecimal characters,
    /// and accumulates the value of the hexadecimal digits into a u32.
    ///
    /// # Arguments
    /// * `s` - A byte slice potentially containing hexadecimal digits
    ///
    /// # Returns
    /// A u32 representing the hexadecimal value of the digits found in the input
    #[inline(always)]
    fn hexadecimal(s: &[u8]) -> u32 {
        s.iter().fold(0u32, |acc, &c| {
            if c.is_ascii_hexdigit() {
                acc * 16 + Self::hex_val(c)
            } else {
                acc // Ignore non-hexadecimal characters
            }
        })
    }

    /// Translates XML escape sequences to their UTF-8 representations.
    ///
    /// This method handles three types of escape sequences:
    /// - Named entities (e.g., `&amp;`, `&lt;`) using the predefined entities map
    /// - Decimal character references (e.g., `&#65;`)
    /// - Hexadecimal character references (e.g., `&#x41;`)
    ///
    /// The method processes the escape sequence starting after the '&' character
    /// and writes the UTF-8 bytes to the specified position in the XML buffer.
    ///
    /// # Arguments
    /// * `from` - Starting position after the '&' character
    /// * `to` - Position where the translated UTF-8 bytes should be written
    ///
    /// # Returns
    /// `Some((next_from, next_to))` if translation succeeds, where `next_from` is
    /// the position after the semicolon and `next_to` is the position after the
    /// written UTF-8 bytes. Returns `None` if the escape sequence is invalid.
    #[inline]
    fn translate_sequence(&mut self, from: XmlIdx, to: XmlIdx) -> Option<(XmlIdx, XmlIdx)> {
        let end = self.scan_until_char(from, SEMI_COLON)?;
        let mut from = from;

        let number = self.xml[from as usize] == HASH;
        if number {
            from += 1;
        }
        let hex_number = self.xml[from as usize] == X_CHAR;
        if hex_number {
            from += 1;
        }

        if from == end {
            // No content between '&' and ';'
            return None;
        }

        let from_slice = &self.xml[from as usize..end as usize];

        let bytes = if number {
            char::from_u32(if hex_number {
                Self::hexadecimal(from_slice)
            } else {
                Self::decimal(from_slice)
            })
            .map(|val| val.to_string().into_bytes())?
        } else {
            ENTITIES_MAP.get(from_slice).map(|entity| entity.to_vec())?
        };

        let buf = &mut self.xml[to as usize..];
        let len = bytes.len().min(buf.len());
        buf[..len].copy_from_slice(&bytes[..len]);

        Some((end + 1, to + len as XmlIdx)) // pass the semicolon
    }

    /// Processes XML content by translating escape sequences in-place.
    ///
    /// This method scans through the specified range looking for '&' characters
    /// that indicate escape sequences. When found, it translates the sequences
    /// to their UTF-8 representations and compacts the content by moving it
    /// forward in the buffer, effectively replacing sequences with their values.
    ///
    /// # Arguments
    /// * `range` - The byte range in the XML buffer to process
    ///
    /// # Returns
    /// `Some((start, end))` where `start` is the original start position and
    /// `end` is the new end position after sequence translation and compaction.
    /// Returns `None` if processing fails.
    #[inline(always)]
    fn parse_pcdata(&mut self, range: &XmlRange) -> XmlRange {
        let end = range.end;
        let mut to = range.start;
        let mut from = range.start;

        loop {
            #[cfg(feature = "parse_escapes")]
            let next_pos = match self.scan_range_for_chartype(from..end, Chartype::ParsePCData) {
                Some(pos) => pos,
                None => end, // No more characters of the specified type found
            };

            #[cfg(not(feature = "parse_escapes"))]
            let next_pos = match self.scan_range_for_char(from..end, CARRIAGE_RETURN) {
                Some(pos) => pos,
                None => end,
            };

            if next_pos >= end {
                if from != to {
                    // Move the tail content to the `to` position
                    self.xml
                        .copy_within(from as usize..end as usize, to as usize);
                }
                to += end - from;
                break;
            }

            if next_pos > from {
                // Move the content before to the `to` position
                self.xml
                    .copy_within(from as usize..next_pos as usize, to as usize);
                to += next_pos - from;
            }

            #[cfg(feature = "parse_escapes")]
            if self.xml[next_pos as usize] == AMPERSAND {
                match self.translate_sequence(next_pos + 1, to) {
                    Some((new_from, new_to)) => {
                        from = new_from;
                        to = new_to;
                    }
                    None => {
                        // Invalid escape sequence, just skip the '&'
                        from += 1;
                    }
                }
            }

            #[cfg(not(feature = "parse_escapes"))]
            {
                // This is a carriage return
                self.xml[to as usize] = NEWLINE; // Replace with a newline character
                to += 1; // Move the `to` position forward
                from = next_pos
                    + if (next_pos + 1) < end && self.xml[(next_pos + 1) as usize] == NEWLINE {
                        2 // Move past the newline character if present
                    } else {
                        1
                    };
            }
        }

        range.start..to
    }

    /// Normalizes attribute values by removing unnecessary whitespace and escape sequences.
    ///
    /// This method scans through the specified range, looking for '&' characters
    /// that indicate escape sequences. It replaces these sequences with their
    /// UTF-8 representations and ensures that the attribute value is compacted
    /// without leading or trailing whitespace.
    ///
    /// # Arguments
    /// * `range` - The byte range in the XML buffer representing the attribute value
    ///
    /// # Returns
    /// A new `XmlLocation` representing the normalized attribute value, with leading
    /// and trailing whitespace removed, and escape sequences translated.
    ///
    /// # Note
    /// This method modifies the XML buffer in place, so the original range may be adjusted.
    #[inline(always)]
    fn normalize_attribute_value(&mut self, range: &XmlRange) -> XmlRange {
        let end = range.end;
        let mut to = range.start;
        let mut from = range.start;
        let mut space_added = false;

        loop {
            let next_pos = match self.scan_range_for_chartype(from..end, Chartype::ParseAtrNorm) {
                Some(pos) => pos,
                None => end, // No more characters of the specified type found
            };

            if next_pos >= end {
                if from != to {
                    // Move the tail content to the `to` position
                    self.xml
                        .copy_within(from as usize..end as usize, to as usize);
                }
                to += end - from;
                if to > range.start && self.xml[(to - 1) as usize] == SPACE {
                    // If we added a space, we need to adjust the end position
                    to -= 1; // Remove the last added space
                }
                break; // No more '&' or whitespace found
            }

            if next_pos > from {
                // Move the content before the '&' or whitespace to the `to` position
                self.xml
                    .copy_within(from as usize..next_pos as usize, to as usize);
                to += next_pos - from;
                space_added = false; // Reset space added flag
            }
            if self.xml[next_pos as usize] == AMPERSAND {
                match self.translate_sequence(next_pos + 1, to) {
                    Some((new_from, new_to)) => {
                        from = new_from;
                        to = new_to;
                    }
                    None => {
                        // Invalid escape sequence, just skip the '&'
                        from += 1;
                    }
                }
                space_added = false; // Reset space added flag
            } else {
                // Handle whitespace normalization
                if !space_added && to != range.start {
                    // Add a space if not already added
                    self.xml[to as usize] = SPACE;
                    to += 1;
                    space_added = true;
                }
                from = next_pos + 1; // Move past the whitespace
            }
        }

        range.start..to
    }

    /// Checks if a byte is of a specific character type.
    ///
    /// This method uses a precomputed table to determine if the byte
    /// belongs to a specific character type (e.g., whitespace, letter, digit).
    ///
    /// # Arguments
    /// * `byte` - The byte to check
    /// * `chartype` - The character type to check against
    ///
    /// # Returns
    /// `true` if the byte matches the character type, `false` otherwise
    #[inline(always)]
    pub(crate) fn is_of_type(byte: u8, chartype: Chartype) -> bool {
        (CHARTYPE_TABLE[byte as usize] & chartype as u8) != 0
    }

    /// Trims trailing whitespace characters from the end of a specified XML range.
    ///
    /// This method iterates backward from the end of the range, removing any
    /// whitespace characters (spaces, tabs, newlines) until it reaches a non-whitespace
    /// character or the start of the range. It effectively adjusts the end index
    /// of the range to exclude trailing whitespace.
    /// # Arguments
    /// * `range` - The byte range to trim
    ///
    /// # Returns
    /// The new end index of the range after trimming trailing whitespace
    #[inline(always)]
    fn trim_the_ending_whitespaces(&self, range: XmlRange) -> XmlIdx {
        let mut end = range.end;
        while end > range.start && Self::is_of_type(self.xml[(end - 1) as usize], Chartype::Space) {
            end -= 1;
        }
        end
    }

    #[cfg(feature = "namespace_removal")]
    /// Removes the namespace prefix from an XML element or attribute name.
    ///
    /// This method searches for a colon (`:`) character within the specified range
    /// and removes everything before it (including the colon itself), effectively
    /// stripping the namespace prefix. If no colon is found, the original range
    /// is returned unchanged.
    ///
    /// # Arguments
    /// * `range` - The byte range containing the name with potential namespace prefix
    ///
    /// # Returns
    /// A new range containing only the local name part (without namespace prefix)
    ///
    /// # Examples
    /// - `prefix:localname` becomes `localname`
    /// - `localname` remains `localname` (no change if no prefix exists)
    #[inline]
    fn remove_namespace_prefix(&mut self, range: XmlRange) -> XmlRange {
        let colon_pos = search_char!(COLON, &self.xml[range.start as usize..range.end as usize])
            .map_or(range.end, |pos| range.start + pos as XmlIdx);

        if colon_pos < range.end {
            // Remove the namespace prefix
            self.xml.copy_within(
                (colon_pos + 1) as usize..range.end as usize,
                range.start as usize,
            );
            range.start..(range.start + (range.end - colon_pos - 1))
        } else {
            range.start..range.end
        }
    }

    /// Parses the XML document and builds the document tree structure.
    ///
    /// This is the main parsing method that implements a state machine to process
    /// XML content. It handles:
    /// - Opening and closing tags
    /// - Self-closing tags
    /// - Attributes with quoted values
    /// - Text content with entity translation
    /// - Comments and processing instructions (bypass)
    /// - CDATA sections (bypass)
    /// - DOCTYPE and DTD declarations (bypass)
    ///
    /// The parser maintains a current parent node and builds the tree by adding
    /// child nodes as it encounters different XML constructs. It performs
    /// validation to ensure well-formed XML structure.
    ///
    /// # Returns
    /// `Ok(())` if parsing succeeds, or a `ParseXmlError` if the XML is malformed
    /// or contains invalid constructs
    ///
    /// # Errors
    /// Returns errors for various malformed XML conditions such as:
    /// - Mismatched opening and closing tags
    /// - Invalid character sequences
    /// - Malformed attributes
    /// - Unexpected end of document
    #[allow(clippy::too_many_lines)]
    pub(crate) fn parse(&mut self) -> Result<(), ParseXmlError> {
        let mut parenthood = Vec::<Parent>::with_capacity(20);
        let mut level = 0usize;

        let mut state = State::Start;
        let mut i: XmlIdx = 0 as XmlIdx;

        let size = self.xml.len() as XmlIdx;

        loop {
            state = match state {
                State::Start => {
                    i = match self.scan_until_char(i, LESS_THAN) {
                        Some(new_i) => new_i,
                        None => break,
                    };
                    i += 1;
                    if i >= size {
                        break;
                    }

                    State::ReadStartOfTag
                }
                State::ReadStartOfTag => {
                    match self.xml[i as usize] {
                        SLASH => {
                            i += 1;
                            State::ReadTagClose
                        }
                        EXCLAMATION_MARK => {
                            i += 1;
                            if i < size {
                                if self.xml[i as usize..].starts_with(b"--") {
                                    i += 2;
                                    i = match self.skip_after_slice(i, 5000, b"-->".as_slice()) {
                                        Some(new_i) => new_i,
                                        None => break,
                                    };
                                } else if self.xml[i as usize..].starts_with(b"DOCTYPE") {
                                    i += 7;
                                    i = match self.scan_until_one_of_2_chars(
                                        i,
                                        GREATER_THAN,
                                        LEFT_BRACKET,
                                    ) {
                                        Some(new_i) => new_i,
                                        None => break,
                                    };

                                    if self.xml[i as usize] == LEFT_BRACKET {
                                        i = match self.scan_until_char(i, RIGHT_BRACKET) {
                                            Some(new_i) => new_i,
                                            None => break,
                                        };
                                        i += 1; // skip ']'
                                        i = match self.skip_chartype(i, Chartype::Space) {
                                            Some(new_i) => new_i,
                                            None => break,
                                        };

                                        if self.xml[i as usize] == GREATER_THAN {
                                            i += 1; // skip '>'
                                        } else {
                                            return self.invalid(
                                                "Expected '>' after DOCTYPE declaration",
                                                i,
                                            );
                                        }
                                    }
                                    i += 1; // skip '>'
                                } else if self.xml[i as usize..].starts_with(b"[CDATA[") {
                                    i += 7;
                                    i = match self.skip_after_slice(i, 5000, b"]]>".as_slice()) {
                                        Some(new_i) => new_i,
                                        None => break,
                                    };
                                } else {
                                    break;
                                }
                            }
                            if i >= size {
                                State::End
                            } else {
                                State::ReadPCData
                            }
                        }
                        QUESTION_MARK => {
                            i += 1;
                            i = match self.skip_after_slice(i, 500, b"?>".as_slice()) {
                                Some(new_i) => new_i,
                                None => break,
                            };
                            if i >= size {
                                State::End
                            } else {
                                State::ReadPCData
                            }
                        }
                        _ => State::ReadTagOpen,
                    }
                }
                State::ReadTagOpen => {
                    let start = i;
                    if !Self::is_of_type(self.xml[i as usize], Chartype::StartSymBol) {
                        return self.invalid(
                            "Tag name must start with a letter or underscore",
                            i as XmlIdx,
                        );
                    }
                    i += 1; // skip first char of tag name
                    i = match self.skip_chartype(i, Chartype::Symbol) {
                        Some(new_i) => new_i,
                        None => break,
                    };

                    #[cfg(feature = "namespace_removal")]
                    // Remove namespace prefix from attribute name
                    let name_range = self.remove_namespace_prefix(start..i);

                    #[cfg(not(feature = "namespace_removal"))]
                    // If namespace removal is not enabled, use the original range
                    let name_range = start..i;

                    #[cfg(feature = "use_cstr")]
                    {
                        // Save the byte that could be overriden by the null terminator
                        let byte = self.xml[i as usize];

                        self.xml[name_range.end as usize] = 0; // Null-terminate the string
                        let node_idx = if level == 0 {
                            // If this is the root element, we set the root node index
                            self.add_node(
                                0,
                                0,
                                NodeType::Element {
                                    name: name_range.start,
                                    attributes: 0..0, // Placeholder for attributes range
                                },
                            )?
                        } else {
                            self.add_node(
                                parenthood[level - 1].parent_idx,
                                parenthood[level - 1].last_child_idx,
                                NodeType::Element {
                                    name: name_range.start,
                                    attributes: 0..0, // Placeholder for attributes range
                                },
                            )?
                        };
                        if level > 0 {
                            parenthood[level - 1].last_child_idx = node_idx;
                        }
                        parenthood.push(Parent::new(node_idx));
                        level += 1;

                        i += 1; // skip the null terminator (or not if there was a removed namespace prefix)

                        if byte == SLASH {
                            if i >= size || self.xml[i as usize] != GREATER_THAN {
                                return self
                                    .invalid("Expected '>' after '/' in self-closing tag", i);
                            }
                            parenthood.pop();
                            level -= 1;

                            if level == 0 {
                                state = State::End;
                                continue;
                            }

                            i += 1;
                            if i >= size {
                                break;
                            }
                            state = State::ReadPCData;
                            continue;
                        } else if byte == GREATER_THAN {
                            if i >= size {
                                break;
                            }
                            state = State::ReadPCData;
                            continue;
                        }
                    }

                    #[cfg(not(feature = "use_cstr"))]
                    {
                        let node_idx = if level == 0 {
                            self.add_node(
                                0,
                                0,
                                NodeType::Element {
                                    name: name_range,
                                    attributes: 0..0, // Placeholder for attributes range
                                },
                            )?
                        } else {
                            self.add_node(
                                parenthood[level - 1].parent_idx,
                                parenthood[level - 1].last_child_idx,
                                NodeType::Element {
                                    name: name_range,
                                    attributes: 0..0, // Placeholder for attributes range
                                },
                            )?
                        };
                        if level > 0 {
                            parenthood[level - 1].last_child_idx = node_idx;
                        }
                        parenthood.push(Parent::new(node_idx));
                        level += 1;
                    }

                    State::ReadAttribute
                }
                State::ReadTagClose => {
                    let start = i;

                    i = match self.scan_until_chartype(i, Chartype::ParseCloseTag) {
                        Some(new_i) => new_i,
                        None => break,
                    };

                    if i == start {
                        return self.invalid("Expected tag name after '</'", i);
                    }

                    #[cfg(feature = "namespace_removal")]
                    // Remove namespace prefix from attribute name
                    let name_range = self.remove_namespace_prefix(start..i);

                    #[cfg(not(feature = "namespace_removal"))]
                    // If namespace removal is not enabled, use the original range
                    let name_range = start..i;

                    let is_greater_than = self.xml[i as usize] == GREATER_THAN;

                    #[cfg(feature = "use_cstr")]
                    {
                        self.xml[name_range.end as usize] = 0; // Null-terminate the string
                        if level > 0 {
                            self.check_closing_tag(
                                parenthood[level - 1].parent_idx,
                                name_range.start,
                            )?;
                        } else {
                            return self.invalid("No opening tag for closing tag", i);
                        }
                    }

                    #[cfg(not(feature = "use_cstr"))]
                    {
                        if level > 0 {
                            self.check_closing_tag(parenthood[level - 1].parent_idx, name_range)?;
                        } else {
                            return self.invalid("No opening tag for closing tag", i);
                        }
                    }

                    if level > 0 {
                        parenthood.pop();
                        level -= 1;
                    }

                    if !is_greater_than {
                        i = match self.scan_until_char(i + 1, GREATER_THAN) {
                            Some(new_i) => new_i,
                            None => break,
                        };
                    }

                    i += 1;
                    if i >= size || level == 0 {
                        State::End
                    } else {
                        State::ReadPCData
                    }
                }
                State::ReadAttribute => {
                    i = match self.skip_chartype(i, Chartype::Space) {
                        Some(new_i) => new_i,
                        None => break,
                    };
                    match self.xml[i as usize] {
                        SLASH => {
                            i += 1;
                            if i >= size {
                                break;
                            }
                            if self.xml[i as usize] != GREATER_THAN {
                                return self
                                    .invalid("Expected '>' after '/' in self-closing tag", i);
                            }
                            parenthood.pop();
                            level -= 1;

                            if level == 0 {
                                State::End
                            } else {
                                i += 1;
                                if i >= size {
                                    break;
                                }
                                State::ReadPCData
                            }
                        }
                        GREATER_THAN => {
                            i += 1;
                            if i >= size {
                                break;
                            }
                            State::ReadPCData
                        }
                        _ => {
                            let start = i;
                            if !Self::is_of_type(self.xml[i as usize], Chartype::StartSymBol) {
                                return self.invalid(
                                    "Attribute name must start with a letter or underscore",
                                    i,
                                );
                            }
                            i += 1;
                            i = match self.skip_chartype(i, Chartype::Symbol) {
                                Some(new_i) => new_i,
                                None => break,
                            };

                            if self.xml[i as usize] != EQUAL {
                                return self.invalid("Attribute must have an '=' sign", i);
                            }
                            let end = i;
                            i += 1; //skip =
                            if i >= size {
                                break;
                            }
                            let quote = self.xml[i as usize];
                            if (quote != b'\'') && (quote != b'"') {
                                return self
                                    .invalid("Attribute value must be enclosed in quotes", i);
                            }
                            i += 1;
                            let value_start = i;
                            i = match self.scan_until_char(i, quote) {
                                Some(new_i) => new_i,
                                None => break,
                            };

                            let value_range = self.normalize_attribute_value(&(value_start..i));

                            #[cfg(feature = "namespace_removal")]
                            // Remove namespace prefix from attribute name
                            let name_range = self.remove_namespace_prefix(start..end);

                            #[cfg(not(feature = "namespace_removal"))]
                            // If namespace removal is not enabled, use the original range
                            let name_range = start..end;

                            #[cfg(feature = "use_cstr")]
                            {
                                self.xml[name_range.end as usize] = 0; // Null-terminate the string
                                self.xml[value_range.end as usize] = 0; // Null-terminate the value
                                if level > 0 {
                                    self.add_attribute(
                                        parenthood[level - 1].parent_idx,
                                        name_range.start,
                                        value_range.start,
                                    )?;
                                } else {
                                    break;
                                }
                            }

                            #[cfg(not(feature = "use_cstr"))]
                            if level > 0 {
                                self.add_attribute(
                                    parenthood[level - 1].parent_idx,
                                    name_range,
                                    value_range,
                                )?;
                            } else {
                                break;
                            }

                            i += 1;
                            State::ReadAttribute
                        }
                    }
                }
                State::ReadPCData => {
                    let space_start = i; // in case we must keep whitespaces
                    match self.skip_chartype(i, Chartype::Space) {
                        Some(new_i) => {
                            i = new_i;
                            if i >= size {
                                State::End
                            } else {
                                let mut start = i;
                                i = match self.scan_until_char(i, LESS_THAN) {
                                    Some(new_i) => new_i,
                                    None => break,
                                };

                                if i > start {
                                    let mut the_end = i;

                                    if cfg!(feature = "trim_pcdata") {
                                        the_end = self.trim_the_ending_whitespaces(start..the_end);
                                    } else {
                                        start = space_start; // Reset start to space_start if not trimming
                                    }

                                    let text_range = self.parse_pcdata(&(start..the_end));

                                    #[cfg(feature = "use_cstr")]
                                    {
                                        self.xml[text_range.end as usize] = 0; // Null-terminate the string
                                        if level > 0 {
                                            let node_idx = self.add_node(
                                                parenthood[level - 1].parent_idx,
                                                parenthood[level - 1].last_child_idx,
                                                NodeType::Text(text_range.start),
                                            )?;
                                            parenthood[level - 1].last_child_idx = node_idx;
                                        } else {
                                            break;
                                        }
                                    }

                                    #[cfg(not(feature = "use_cstr"))]
                                    if level > 0 {
                                        let node_idx = self.add_node(
                                            parenthood[level - 1].parent_idx,
                                            parenthood[level - 1].last_child_idx,
                                            NodeType::Text(text_range),
                                        )?;
                                        parenthood[level - 1].last_child_idx = node_idx;
                                    } else {
                                        break;
                                    }
                                } else {
                                    #[cfg(feature = "keep_ws_only_pcdata")]
                                    if i > space_start && level != 0 {
                                        #[cfg(feature = "use_cstr")]
                                        {
                                            self.xml[i as usize] = 0; // Null-terminate the string
                                            if level > 0 {
                                                let node_idx = self.add_node(
                                                    parenthood[level - 1].parent_idx,
                                                    parenthood[level - 1].last_child_idx,
                                                    NodeType::Text(space_start),
                                                )?;
                                                parenthood[level - 1].last_child_idx = node_idx;
                                            } else {
                                                break;
                                            }
                                        }

                                        #[cfg(not(feature = "use_cstr"))]
                                        if level > 0 {
                                            // If we are keeping whitespace-only text nodes
                                            let node_idx = self.add_node(
                                                parenthood[level - 1].parent_idx,
                                                parenthood[level - 1].last_child_idx,
                                                NodeType::Text(space_start..i),
                                            )?;
                                            parenthood[level - 1].last_child_idx = node_idx;
                                        } else {
                                            break;
                                        }
                                    }
                                }

                                i += 1; // Reset i to the position after the '<'
                                if i >= size {
                                    break;
                                }
                                State::ReadStartOfTag
                            }
                        }
                        None => State::End,
                    }
                }
                State::End => {
                    return Ok(());
                }
            };
        }

        Err(ParseXmlError::InvalidXml(
            "Unexpected end of XML document.".to_string(),
        ))
    }
}
