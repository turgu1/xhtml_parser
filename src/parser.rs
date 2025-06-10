//! Parser for XML documents.
//!
//! This module provides functionality to parse XML content, handling various node types and attributes.
//! It defines the `parser()` method for processing XML data.

#![allow(unused_macros)]

use crate::defs::{NodeIdx, ParseXmlError, XmlIdx, XmlRange};
use crate::document::Document;
use crate::node_type::NodeType;

use kmp::kmp_find;
use phf::phf_map;

enum State {
    Start,
    ReadTag,
    ReadTagOpen,
    ReadTagClose,
    ReadAttribute,
    ReadContent,
    End,
}

const LESS_THAN: u8 = '<' as u8;
const GREATER_THAN: u8 = '>' as u8;
const SLASH: u8 = '/' as u8;
const EQUAL: u8 = '=' as u8;
const EXCLAMATION_MARK: u8 = '!' as u8;
const QUESTION_MARK: u8 = '?' as u8;
const AMPERSAND: u8 = '&' as u8;
const SEMI_COLON: u8 = ';' as u8;
const HASH: u8 = '#' as u8;
const X_CHAR: u8 = 'x' as u8;
const COLON: u8 = ':' as u8;

#[allow(dead_code)]
#[derive(Clone, Copy)]
enum Chartype {
    ParsePcData = 1,   // \0, &, \r, <
    ParseAttr = 2,     // \0, &, \r, ', "
    ParseAttrWs = 4,   // \0, &, \r, ', ", \n, tab
    Space = 8,         // \r, \n, space, tab
    ParseCData = 16,   // \0, ], >, \r
    ParseComment = 32, // \0, -, >, \r
    Symbol = 64,       // Any symbol > 127, a-z, A-Z, 0-9, _, :, -, .
    StartSymBol = 128, // Any symbol > 127, a-z, A-Z, _, :
}

//const SPACE_AND_CLOSE_SIGN: u8 = Chartype::Space as u8 | Chartype::ParseCData as u8;
const SYMBOL: u8 = Chartype::Symbol as u8;
const START_SYMBOL: u8 = Chartype::StartSymBol as u8;

const CHARTYPE_TABLE: [u8; 256] = [
    55, 0, 0, 0, 0, 0, 0, 0, 0, 12, 12, 0, 0, 63, 0, 0, // 0-15
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 16-31
    8, 0, 6, 0, 0, 0, 7, 6, 0, 0, 0, 0, 0, 96, 64, 0, // 32-47
    64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 192, 0, 1, 0, 48, 0, // 48-63
    0, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, // 64-79
    192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 0, 0, 16, 0, 192, // 80-95
    0, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, // 96-111
    192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 0, 0, 0, 0, 0, // 112-127
    192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, // 128+
    192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192,
    192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192,
    192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192,
    192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192,
    192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192,
    192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192, 192,
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

// In the following macros, the code is embedded within a block ( {{..}} ) to have have a local scope for local variable.
// This allows us to use the some variable names in different macros or other code without conflicts.

macro_rules! skip_after_slice_nobreak {
    ($contents: expr, $p: ident, $max: expr, $slice: expr) => {{
        if $p < $contents.len() as XmlIdx {
            let max_pos: XmlIdx = ($p + $max).min($contents.len() as XmlIdx);
            if let Some(pos) = kmp_find($slice, &$contents[$p as usize..max_pos as usize]) {
                $p += pos as XmlIdx + $slice.len() as XmlIdx;
            } else {
                $p = $contents.len() as XmlIdx; // Move to the end if no more characters match
            }
        }
    }};
}
macro_rules! skip_chartype {
    ($contents: expr, $p: ident, $chartype: expr) => {{
        if $p >= $contents.len() as XmlIdx {
            break;
        }
        if let Some(pos) = (&$contents[$p as usize..])
            .iter()
            .position(|&c| (CHARTYPE_TABLE[c as usize] & $chartype as u8) == 0)
        {
            $p += pos as XmlIdx;
        } else {
            break;
        }
    }};
}

macro_rules! skip_chartype_nobreak {
    ($contents: expr, $p: ident, $chartype: expr) => {{
        if $p < $contents.len() as XmlIdx {
            if let Some(pos) = (&$contents[$p as usize..])
                .iter()
                .position(|&c| (CHARTYPE_TABLE[c as usize] & $chartype as u8) == 0)
            {
                $p += pos as XmlIdx;
            } else {
                $p = $contents.len() as XmlIdx; // Move to the end if no more characters match
            }
        } else {
            $p = $contents.len() as XmlIdx; // Move to the end if pointer is out of bounds
        }
    }};
}

macro_rules! scan_until_chartype {
    ($contents: expr, $p: ident, $chartype: expr) => {{
        if $p >= $contents.len() as XmlIdx {
            break;
        }
        if let Some(pos) = (&$contents[$p..])
            .iter()
            .position(|&c| (CHARTYPE_TABLE[c as usize] & $chartype as u8) != 0)
        {
            $p += pos;
        } else {
            break;
        }
    }};
}

macro_rules! scan_until_char_or_nochange {
    ($contents: expr, $p: ident, $char: ident) => {{
        if $p < $contents.len() as XmlIdx {
            if let Some(pos) = (&$contents[$p as usize..]).iter().position(|&c| c == $char) {
                $p += pos as XmlIdx;
            }
        }
    }};
}

macro_rules! scan_until_char_nobreak {
    ($contents: expr, $p: ident, $char: ident) => {{
        if $p < $contents.len() as XmlIdx {
            if let Some(pos) = (&$contents[$p as usize..]).iter().position(|&c| c == $char) {
                $p += pos as XmlIdx;
            } else {
                $p = $contents.len() as XmlIdx; // Move to the end if no more characters match
            }
        } else {
            $p = $contents.len() as XmlIdx; // Move to the end if pointer is out of bounds
        }
    }};
}

macro_rules! scan_until_char {
    ($contents: expr, $p: ident, $char: ident) => {{
        if $p >= $contents.len() as XmlIdx {
            break;
        }
        if let Some(pos) = (&$contents[$p as usize..]).iter().position(|&c| c == $char) {
            $p += pos as XmlIdx;
        } else {
            break;
        }
    }};
}

impl Document {
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
        let start = if pos > 30 { pos - 30 } else { 0 };
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
    #[inline]
    fn check_closing_tag(&self, parent_idx: NodeIdx, range: XmlRange) -> Result<(), ParseXmlError> {
        let parent = self.get_node(parent_idx)?;
        if let NodeType::Element { name, .. } = parent.get_node_type() {
            let tag_name = self.get_str_from_range(name);
            let closing_tag = self.get_str_from_range(&range);
            if tag_name != closing_tag {
                return self.invalid(
                    &format!(
                        "Closing tag '{}' does not match opening tag '{}'",
                        closing_tag, tag_name
                    ),
                    range.start,
                );
            }
        } else {
            return self.invalid("Expected an element node for closing tag", range.start);
        }
        Ok(())
    }

    /// Converts a byte slice containing ASCII decimal digits to a String.
    ///
    /// This method filters out any non-digit characters and collects only
    /// the valid ASCII digits (0-9) into a string representation.
    ///
    /// # Arguments
    /// * `s` - A byte slice potentially containing decimal digits
    ///
    /// # Returns
    /// A String containing only the decimal digits found in the input
    #[inline]
    fn decimal(&self, s: &[u8]) -> String {
        // Convert a byte slice containing ASCII digits to a String
        // This function assumes that the input is valid ASCII digits (0-9).
        // Using `collect` to gather the digits into a String
        s.iter()
            .filter(|&&c| c.is_ascii_digit())
            .map(|&c| c as char)
            .collect::<String>()
    }

    /// Converts a byte slice containing hexadecimal digits to a String.
    ///
    /// This method filters out any non-hexadecimal characters and collects only
    /// the valid ASCII hexadecimal digits (0-9, A-F, a-f) into a string representation.
    ///
    /// # Arguments
    /// * `s` - A byte slice potentially containing hexadecimal digits
    ///
    /// # Returns
    /// A String containing only the hexadecimal digits found in the input
    #[inline]
    fn hexadecimal(&self, s: &[u8]) -> String {
        // Convert a byte slice containing hexadecimal digits to a String
        // This function assumes that the input is valid hexadecimal digits (0-9, A-F, a-f).
        s.iter()
            .filter(|&&c| c.is_ascii_hexdigit())
            .map(|&c| c as char)
            .collect::<String>()
    }

    /// Translates XML entities to their UTF-8 representations.
    ///
    /// This method handles three types of entity references:
    /// - Named entities (e.g., `&amp;`, `&lt;`) using the predefined entities map
    /// - Decimal character references (e.g., `&#65;`)
    /// - Hexadecimal character references (e.g., `&#x41;`)
    ///
    /// The method processes the entity reference starting after the '&' character
    /// and writes the UTF-8 bytes to the specified position in the XML buffer.
    ///
    /// # Arguments
    /// * `from` - Starting position after the '&' character
    /// * `to` - Position where the translated UTF-8 bytes should be written
    ///
    /// # Returns
    /// `Some((next_from, next_to))` if translation succeeds, where `next_from` is
    /// the position after the semicolon and `next_to` is the position after the
    /// written UTF-8 bytes. Returns `None` if the entity is invalid.
    #[inline]
    fn translate_entity(&mut self, from: XmlIdx, to: XmlIdx) -> Option<(XmlIdx, XmlIdx)> {
        let mut from = from;
        let number = self.xml[from as usize] == HASH;
        if number {
            from += 1;
        }
        let hex_number = self.xml[from as usize] == X_CHAR;
        if hex_number {
            from += 1;
        }
        let start = from;
        scan_until_char_or_nochange!(self.xml, from, SEMI_COLON);
        if from == start {
            return None; // No entity found
        }

        let from_u8 = &self.xml[start as usize..from as usize];
        let bytes = if number {
            let value = if hex_number {
                u32::from_str_radix(self.hexadecimal(from_u8).as_str(), 16).unwrap_or(0)
            } else {
                self.decimal(from_u8).parse::<u32>().unwrap_or(0)
            };
            if let Some(val) = char::from_u32(value) {
                val.to_string().into_bytes()
            } else {
                // Invalid character
                return None;
            }
        } else {
            if let Some(entity) = ENTITIES_MAP.get(from_u8) {
                entity.to_vec()
            } else {
                return None;
            }
        };
        let buf = &mut self.xml[to as usize..];
        let len = bytes.len().min(buf.len());
        buf[..len].copy_from_slice(&bytes[..len]);

        Some((from + 1, to + len as XmlIdx)) // pass the semicolon
    }

    /// Processes XML content by translating entity references in-place.
    ///
    /// This method scans through the specified range looking for '&' characters
    /// that indicate entity references. When found, it translates the entities
    /// to their UTF-8 representations and compacts the content by moving it
    /// forward in the buffer, effectively replacing entities with their values.
    ///
    /// # Arguments
    /// * `range` - The byte range in the XML buffer to process
    ///
    /// # Returns
    /// `Some((start, end))` where `start` is the original start position and
    /// `end` is the new end position after entity translation and compaction.
    /// Returns `None` if processing fails.
    #[inline]
    fn parse_escapes(&mut self, range: &XmlRange) -> Option<(XmlIdx, XmlIdx)> {
        let end = range.end;
        let mut to = range.start;
        let mut from = range.start;

        loop {
            let ampersand_pos = self.xml[from as usize..end as usize]
                .iter()
                .position(|&c| c == AMPERSAND)
                .map_or(end, |pos| from + pos as XmlIdx);
            if ampersand_pos >= end {
                if from != to {
                    // Move the tail content to the `to` position
                    self.xml
                        .copy_within(from as usize..end as usize, to as usize);
                }
                to += end - from;
                break; // No more '&' found
            } else {
                if ampersand_pos > from {
                    // Move the content before the '&' to the `to` position
                    self.xml
                        .copy_within(from as usize..ampersand_pos as usize, to as usize);
                    to += ampersand_pos - from;
                }
                if let Some((new_from, new_to)) = self.translate_entity(ampersand_pos + 1, to) {
                    from = new_from;
                    to = new_to;
                } else {
                    // Invalid entity, just skip the '&'
                    from += 1;
                }
            }
        }

        Some((range.start, to))
    }

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
        let colon_pos = self.xml[range.start as usize..range.end as usize]
            .iter()
            .position(|&c| c == COLON)
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
    /// - DTD and DTD declarations (bypass)
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
    pub fn parse(&mut self) -> Result<(), ParseXmlError> {
        let root_idx: NodeIdx = 0;

        //let contents = self.xml.as_slice();

        let mut parent_idx = root_idx;
        let mut state = State::Start;
        let mut i: XmlIdx = 0 as XmlIdx;
        let size = self.xml.len() as XmlIdx;

        loop {
            state = match state {
                State::Start => {
                    scan_until_char!(self.xml, i, LESS_THAN);
                    State::ReadTag
                }
                State::ReadTag => {
                    if self.xml[i as usize] != LESS_THAN {
                        return self.invalid("Expected '<' to start a tag", i as XmlIdx);
                    }
                    i += 1; // skip first '<'
                    if i >= size {
                        State::End
                    } else {
                        match self.xml[i as usize] {
                            SLASH => {
                                i += 1;
                                State::ReadTagClose
                            }
                            EXCLAMATION_MARK => {
                                i += 1;
                                if i < size {
                                    if self.xml[i as usize] == b'[' {
                                        i += 1;
                                        skip_after_slice_nobreak!(
                                            self.xml,
                                            i,
                                            5000,
                                            &b">".as_slice()
                                        );
                                    } else if self.xml[i as usize] == b'-' {
                                        i += 1;
                                        skip_after_slice_nobreak!(
                                            self.xml,
                                            i,
                                            5000,
                                            &b"-->".as_slice()
                                        );
                                    } else {
                                        break;
                                    }
                                }
                                skip_chartype_nobreak!(self.xml, i, Chartype::Space);
                                if i >= size {
                                    State::End
                                } else {
                                    State::ReadTag
                                }
                            }
                            QUESTION_MARK => {
                                i += 1;
                                skip_after_slice_nobreak!(self.xml, i, 500, &b"?>".as_slice());
                                skip_chartype_nobreak!(self.xml, i, Chartype::Space);
                                if i >= size {
                                    State::End
                                } else {
                                    State::ReadTag
                                }
                            }
                            _ => State::ReadTagOpen,
                        }
                    }
                }
                State::ReadTagOpen => {
                    let start = i;
                    if CHARTYPE_TABLE[self.xml[i as usize] as usize] & START_SYMBOL == 0 {
                        return self.invalid(
                            "Tag name must start with a letter or underscore",
                            i as XmlIdx,
                        );
                    }
                    i += 1; // skip first char of tag name
                    skip_chartype!(self.xml, i, SYMBOL);

                    let name_range = if cfg!(feature = "namespace_removal") {
                        // Remove namespace prefix from attribute name
                        self.remove_namespace_prefix(start..i)
                    } else {
                        // If namespace removal is not enabled, use the original range
                        start..i
                    };
                    let node_idx = self.add_node(
                        parent_idx,
                        NodeType::Element {
                            name: name_range,
                            attributes: 0..0, // Placeholder for attributes range
                        },
                        start,
                    )?;

                    parent_idx = node_idx;
                    State::ReadAttribute
                }
                State::ReadTagClose => {
                    let start = i;
                    skip_chartype!(self.xml, i, Chartype::Space);
                    scan_until_char_nobreak!(self.xml, i, GREATER_THAN);

                    let name_range = if cfg!(feature = "namespace_removal") {
                        // Remove namespace prefix from attribute name
                        self.remove_namespace_prefix(start..i)
                    } else {
                        // If namespace removal is not enabled, use the original range
                        start..i
                    };

                    self.check_closing_tag(parent_idx, name_range)?;
                    parent_idx = if parent_idx == 1 {
                        0
                    } else {
                        self.get_parent_idx(parent_idx)?
                    };
                    i += 1;
                    // Todo: in an EPub context not to get rid of spaces here
                    skip_chartype_nobreak!(self.xml, i, Chartype::Space);
                    if i >= size {
                        State::End
                    } else {
                        State::ReadContent
                    }
                }
                State::ReadAttribute => {
                    skip_chartype!(self.xml, i, Chartype::Space);
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
                            parent_idx = self.get_parent_idx(parent_idx)?;

                            i += 1;
                            if i >= size {
                                break;
                            }
                            State::ReadContent
                        }
                        GREATER_THAN => {
                            i += 1;
                            if i >= size {
                                break;
                            }
                            State::ReadContent
                        }
                        _ => {
                            let start = i;
                            if CHARTYPE_TABLE[self.xml[i as usize] as usize] & START_SYMBOL == 0 {
                                return self.invalid(
                                    "Attribute name must start with a letter or underscore",
                                    i,
                                );
                            }
                            i += 1;
                            skip_chartype!(self.xml, i, SYMBOL);
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
                            scan_until_char!(self.xml, i, quote);

                            let value_range = if cfg!(feature = "parse_escapes") {
                                // Parse and translate entities in the attribute value
                                if let Some((_new_start, new_end)) =
                                    self.parse_escapes(&(value_start..i))
                                {
                                    value_start..new_end
                                } else {
                                    return self
                                        .invalid("Invalid entity in attribute value", value_start);
                                }
                            } else {
                                // If escape parsing is not enabled, use the original range
                                value_start..i
                            };

                            let name_range = if cfg!(feature = "namespace_removal") {
                                // Remove namespace prefix from attribute name
                                self.remove_namespace_prefix(start..end)
                            } else {
                                // If namespace removal is not enabled, use the original range
                                start..end
                            };

                            self.add_attribute(parent_idx, name_range, value_range)?;

                            i += 1;
                            State::ReadAttribute
                        }
                    }
                }
                State::ReadContent => {
                    // i += 1;
                    // if i >= size {
                    //     break;
                    // }
                    // ToDo: May need to keep spaces
                    skip_chartype!(self.xml, i, Chartype::Space);

                    let start = i;
                    scan_until_char!(self.xml, i, LESS_THAN);
                    if i > start {
                        let text_range = if cfg!(feature = "parse_escapes") {
                            // Parse and translate entities in the content
                            if let Some((_new_start, new_end)) = self.parse_escapes(&(start..i)) {
                                start..new_end
                            } else {
                                return self.invalid("Invalid entity in content", start);
                            }
                        } else {
                            // If escape parsing is not enabled, use the original range
                            start..i
                        };
                        // Add text node with content
                        self.add_node(parent_idx, NodeType::Text(text_range), start)?;
                    }
                    State::ReadTag
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
