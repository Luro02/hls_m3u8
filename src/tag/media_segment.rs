use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;
use std::time::Duration;
use chrono::{DateTime, FixedOffset, NaiveDate};
use trackable::error::ErrorKindExt;

use {Error, ErrorKind, Result};
use attribute::{AttributePairs, DecimalFloatingPoint, QuotedString};
use types::{ByteRange, DecryptionKey, M3u8String, ProtocolVersion, Yes};

/// [4.3.2.1. EXTINF]
///
/// [4.3.2.1. EXTINF]: https://tools.ietf.org/html/rfc8216#section-4.3.2.1
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExtInf {
    duration: Duration,
    title: Option<M3u8String>,
}
impl ExtInf {
    pub(crate) const PREFIX: &'static str = "#EXTINF:";

    /// Makes a new `ExtInf` tag.
    pub fn new(duration: Duration) -> Self {
        ExtInf {
            duration,
            title: None,
        }
    }

    /// Makes a new `ExtInf` tag with the given title.
    pub fn with_title(duration: Duration, title: M3u8String) -> Self {
        ExtInf {
            duration,
            title: Some(title),
        }
    }

    /// Returns the duration of the associated media segment.
    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// Returns the title of the associated media segment.
    pub fn title(&self) -> Option<&M3u8String> {
        self.title.as_ref()
    }

    /// Returns the protocol compatibility version that this tag requires.
    pub fn requires_version(&self) -> ProtocolVersion {
        if self.duration.subsec_nanos() == 0 {
            ProtocolVersion::V1
        } else {
            ProtocolVersion::V3
        }
    }
}
impl fmt::Display for ExtInf {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", Self::PREFIX)?;

        let duration = (self.duration.as_secs() as f64)
            + (self.duration.subsec_nanos() as f64 / 1_000_000_000.0);
        write!(f, "{}", duration)?;

        if let Some(ref title) = self.title {
            write!(f, ",{}", title)?;
        }
        Ok(())
    }
}
impl FromStr for ExtInf {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        track_assert!(s.starts_with(Self::PREFIX), ErrorKind::InvalidInput);
        let mut tokens = s.split_at(Self::PREFIX.len()).1.splitn(2, ',');

        let seconds: DecimalFloatingPoint =
            may_invalid!(tokens.next().expect("Never fails").parse())?;
        let duration = seconds.to_duration();

        let title = if let Some(title) = tokens.next() {
            Some(track!(M3u8String::new(title))?)
        } else {
            None
        };
        Ok(ExtInf { duration, title })
    }
}

/// [4.3.2.2. EXT-X-BYTERANGE]
///
/// [4.3.2.2. EXT-X-BYTERANGE]: https://tools.ietf.org/html/rfc8216#section-4.3.2.2
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExtXByteRange {
    range: ByteRange,
}
impl ExtXByteRange {
    pub(crate) const PREFIX: &'static str = "#EXT-X-BYTERANGE:";

    /// Makes a new `ExtXByteRange` tag.
    pub fn new(range: ByteRange) -> Self {
        ExtXByteRange { range }
    }

    /// Returns the range of the associated media segment.
    pub fn range(&self) -> ByteRange {
        self.range
    }

    /// Returns the protocol compatibility version that this tag requires.
    pub fn requires_version(&self) -> ProtocolVersion {
        ProtocolVersion::V4
    }
}
impl fmt::Display for ExtXByteRange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", Self::PREFIX, self.range)
    }
}
impl FromStr for ExtXByteRange {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        track_assert!(s.starts_with(Self::PREFIX), ErrorKind::InvalidInput);
        let range = may_invalid!(s.split_at(Self::PREFIX.len()).1.parse())?;
        Ok(ExtXByteRange { range })
    }
}

/// [4.3.2.3. EXT-X-DISCONTINUITY]
///
/// [4.3.2.3. EXT-X-DISCONTINUITY]: https://tools.ietf.org/html/rfc8216#section-4.3.2.3
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExtXDiscontinuity;
impl ExtXDiscontinuity {
    pub(crate) const PREFIX: &'static str = "#EXT-X-DISCONTINUITY";

    /// Returns the protocol compatibility version that this tag requires.
    pub fn requires_version(&self) -> ProtocolVersion {
        ProtocolVersion::V1
    }
}
impl fmt::Display for ExtXDiscontinuity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Self::PREFIX.fmt(f)
    }
}
impl FromStr for ExtXDiscontinuity {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        track_assert_eq!(s, Self::PREFIX, ErrorKind::InvalidInput);
        Ok(ExtXDiscontinuity)
    }
}

/// [4.3.2.4. EXT-X-KEY]
///
/// [4.3.2.4. EXT-X-KEY]: https://tools.ietf.org/html/rfc8216#section-4.3.2.4
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExtXKey {
    key: Option<DecryptionKey>,
}
impl ExtXKey {
    pub(crate) const PREFIX: &'static str = "#EXT-X-KEY:";

    /// Makes a new `ExtXKey` tag.
    pub fn new(key: DecryptionKey) -> Self {
        ExtXKey { key: Some(key) }
    }

    /// Makes a new `ExtXKey` tag without a decryption key.
    ///
    /// This tag has the `METHDO=NONE` attribute.
    pub fn new_without_key() -> Self {
        ExtXKey { key: None }
    }

    /// Returns the decryption key for the following media segments and media initialization sections.
    pub fn key(&self) -> Option<&DecryptionKey> {
        self.key.as_ref()
    }

    /// Returns the protocol compatibility version that this tag requires.
    pub fn requires_version(&self) -> ProtocolVersion {
        self.key
            .as_ref()
            .map_or(ProtocolVersion::V1, |k| k.requires_version())
    }
}
impl fmt::Display for ExtXKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", Self::PREFIX)?;
        if let Some(ref key) = self.key {
            write!(f, "{}", key)?;
        } else {
            write!(f, "METHOD=NONE")?;
        }
        Ok(())
    }
}
impl FromStr for ExtXKey {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        track_assert!(s.starts_with(Self::PREFIX), ErrorKind::InvalidInput);
        let suffix = s.split_at(Self::PREFIX.len()).1;

        if AttributePairs::parse(suffix)
            .find(|a| a.as_ref().ok() == Some(&("METHOD", "NONE")))
            .is_some()
        {
            for attr in AttributePairs::parse(suffix) {
                let (key, _) = track!(attr)?;
                track_assert_ne!(key, "URI", ErrorKind::InvalidInput);
                track_assert_ne!(key, "IV", ErrorKind::InvalidInput);
                track_assert_ne!(key, "KEYFORMAT", ErrorKind::InvalidInput);
                track_assert_ne!(key, "KEYFORMATVERSIONS", ErrorKind::InvalidInput);
            }
            Ok(ExtXKey { key: None })
        } else {
            let key = track!(suffix.parse())?;
            Ok(ExtXKey { key: Some(key) })
        }
    }
}

/// [4.3.2.5. EXT-X-MAP]
///
/// [4.3.2.5. EXT-X-MAP]: https://tools.ietf.org/html/rfc8216#section-4.3.2.5
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExtXMap {
    uri: QuotedString,
    range: Option<ByteRange>,
}
impl ExtXMap {
    pub(crate) const PREFIX: &'static str = "#EXT-X-MAP:";

    /// Makes a new `ExtXMap` tag.
    pub fn new(uri: QuotedString) -> Self {
        ExtXMap { uri, range: None }
    }

    /// Makes a new `ExtXMap` tag with the given range.
    pub fn with_range(uri: QuotedString, range: ByteRange) -> Self {
        ExtXMap {
            uri,
            range: Some(range),
        }
    }

    /// Returns the URI that identifies a resource that contains the media initialization section.
    pub fn uri(&self) -> &QuotedString {
        &self.uri
    }

    /// Returns the range of the media initialization section.
    pub fn range(&self) -> Option<ByteRange> {
        self.range
    }

    /// Returns the protocol compatibility version that this tag requires.
    pub fn requires_version(&self) -> ProtocolVersion {
        ProtocolVersion::V6
    }
}
impl fmt::Display for ExtXMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", Self::PREFIX)?;
        write!(f, "URI={}", self.uri)?;
        if let Some(ref x) = self.range {
            write!(f, ",BYTERANGE={}", x)?;
        }
        Ok(())
    }
}
impl FromStr for ExtXMap {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        track_assert!(s.starts_with(Self::PREFIX), ErrorKind::InvalidInput);

        let mut uri = None;
        let mut range = None;
        let attrs = AttributePairs::parse(s.split_at(Self::PREFIX.len()).1);
        for attr in attrs {
            let (key, value) = track!(attr)?;
            match key {
                "URI" => uri = Some(track!(value.parse())?),
                "BYTERANGE" => range = Some(track!(value.parse())?),
                _ => {
                    // [6.3.1. General Client Responsibilities]
                    // > ignore any attribute/value pair with an unrecognized AttributeName.
                }
            }
        }

        let uri = track_assert_some!(uri, ErrorKind::InvalidInput);
        Ok(ExtXMap { uri, range })
    }
}

/// [4.3.2.6. EXT-X-PROGRAM-DATE-TIME]
///
/// [4.3.2.6. EXT-X-PROGRAM-DATE-TIME]: https://tools.ietf.org/html/rfc8216#section-4.3.2.6
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExtXProgramDateTime {
    date_time: DateTime<FixedOffset>,
}
impl ExtXProgramDateTime {
    pub(crate) const PREFIX: &'static str = "#EXT-X-PROGRAM-DATE-TIME:";

    /// Makes a new `ExtXProgramDateTime` tag.
    pub fn new(date_time: DateTime<FixedOffset>) -> Self {
        ExtXProgramDateTime { date_time }
    }

    /// Returns the `DateTime` of the first sample of the associated media segment.
    pub fn date_time(&self) -> DateTime<FixedOffset> {
        self.date_time
    }

    /// Returns the protocol compatibility version that this tag requires.
    pub fn requires_version(&self) -> ProtocolVersion {
        ProtocolVersion::V1
    }
}
impl fmt::Display for ExtXProgramDateTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", Self::PREFIX, self.date_time.to_rfc3339())
    }
}
impl FromStr for ExtXProgramDateTime {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        track_assert!(s.starts_with(Self::PREFIX), ErrorKind::InvalidInput);
        let suffix = s.split_at(Self::PREFIX.len()).1;
        Ok(ExtXProgramDateTime {
            date_time: track!(suffix.parse().map_err(|e| ErrorKind::InvalidInput.cause(e)))?,
        })
    }
}

/// [4.3.2.7.  EXT-X-DATERANGE]
///
/// [4.3.2.7.  EXT-X-DATERANGE]: https://tools.ietf.org/html/rfc8216#section-4.3.2.7
///
/// TODO: Implement properly
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExtXDateRange {
    pub id: QuotedString,
    pub class: Option<QuotedString>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub duration: Option<Duration>,
    pub planned_duration: Option<Duration>,
    pub scte35_cmd: Option<QuotedString>,
    pub scte35_out: Option<QuotedString>,
    pub scte35_in: Option<QuotedString>,
    pub end_on_next: Option<Yes>,
    pub client_attributes: BTreeMap<String, String>,
}
impl ExtXDateRange {
    pub(crate) const PREFIX: &'static str = "#EXT-X-DATERANGE:";

    /// Returns the protocol compatibility version that this tag requires.
    pub fn requires_version(&self) -> ProtocolVersion {
        ProtocolVersion::V1
    }
}
impl fmt::Display for ExtXDateRange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", Self::PREFIX)?;
        write!(f, "ID={}", self.id)?;
        if let Some(ref x) = self.class {
            write!(f, ",CLASS={}", x)?;
        }
        write!(
            f,
            ",START_DATE={:?}",
            self.start_date.format("%Y-%m-%d").to_string()
        )?;
        if let Some(ref x) = self.end_date {
            write!(f, ",END_DATE={:?}", x.format("%Y-%m-%d").to_string())?;
        }
        if let Some(x) = self.duration {
            write!(f, ",DURATION={}", DecimalFloatingPoint::from_duration(x))?;
        }
        if let Some(x) = self.planned_duration {
            write!(
                f,
                ",PLANNED_DURATION={}",
                DecimalFloatingPoint::from_duration(x)
            )?;
        }
        if let Some(ref x) = self.scte35_cmd {
            write!(f, ",SCTE35_CMD={}", x)?;
        }
        if let Some(ref x) = self.scte35_out {
            write!(f, ",SCTE35_OUT={}", x)?;
        }
        if let Some(ref x) = self.scte35_in {
            write!(f, ",SCTE35_IN={}", x)?;
        }
        if let Some(ref x) = self.end_on_next {
            write!(f, ",END_ON_NEXT={}", x)?;
        }
        for (k, v) in &self.client_attributes {
            write!(f, ",{}={}", k, v)?;
        }
        Ok(())
    }
}
impl FromStr for ExtXDateRange {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        track_assert!(s.starts_with(Self::PREFIX), ErrorKind::InvalidInput);

        let mut id = None;
        let mut class = None;
        let mut start_date = None;
        let mut end_date = None;
        let mut duration = None;
        let mut planned_duration = None;
        let mut scte35_cmd = None;
        let mut scte35_out = None;
        let mut scte35_in = None;
        let mut end_on_next = None;
        let mut client_attributes = BTreeMap::new();
        let attrs = AttributePairs::parse(s.split_at(Self::PREFIX.len()).1);
        for attr in attrs {
            let (key, value) = track!(attr)?;
            match key {
                "ID" => id = Some(track!(value.parse())?),
                "CLASS" => class = Some(track!(value.parse())?),
                "START-DATE" => {
                    let s: QuotedString = track!(value.parse())?;
                    start_date = Some(track!(
                        NaiveDate::parse_from_str(s.as_str(), "%Y-%m-%d")
                            .map_err(|e| ErrorKind::InvalidInput.cause(e))
                    )?);
                }
                "END-DATE" => {
                    let s: QuotedString = track!(value.parse())?;
                    end_date = Some(track!(
                        NaiveDate::parse_from_str(s.as_str(), "%Y-%m-%d")
                            .map_err(|e| ErrorKind::InvalidInput.cause(e))
                    )?);
                }
                "DURATION" => {
                    let seconds: DecimalFloatingPoint = track!(value.parse())?;
                    duration = Some(seconds.to_duration());
                }
                "PLANNED-DURATION" => {
                    let seconds: DecimalFloatingPoint = track!(value.parse())?;
                    planned_duration = Some(seconds.to_duration());
                }
                "SCTE35-CMD" => scte35_cmd = Some(track!(value.parse())?),
                "SCTE35-OUT" => scte35_out = Some(track!(value.parse())?),
                "SCTE35-IN" => scte35_in = Some(track!(value.parse())?),
                "END-ON-NEXT" => end_on_next = Some(track!(value.parse())?),
                _ => {
                    if key.starts_with("X-") {
                        client_attributes.insert(key.split_at(2).1.to_owned(), value.to_owned());
                    } else {
                        // [6.3.1. General Client Responsibilities]
                        // > ignore any attribute/value pair with an unrecognized AttributeName.
                    }
                }
            }
        }

        let id = track_assert_some!(id, ErrorKind::InvalidInput);
        let start_date = track_assert_some!(start_date, ErrorKind::InvalidInput);
        if end_on_next.is_some() {
            track_assert!(class.is_some(), ErrorKind::InvalidInput);
        }
        Ok(ExtXDateRange {
            id,
            class,
            start_date,
            end_date,
            duration,
            planned_duration,
            scte35_cmd,
            scte35_out,
            scte35_in,
            end_on_next,
            client_attributes,
        })
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use attribute::HexadecimalSequence;
    use types::EncryptionMethod;
    use super::*;

    #[test]
    fn extinf() {
        let tag = ExtInf::new(Duration::from_secs(5));
        assert_eq!("#EXTINF:5".parse().ok(), Some(tag.clone()));
        assert_eq!(tag.to_string(), "#EXTINF:5");
        assert_eq!(tag.requires_version(), ProtocolVersion::V1);

        let tag = ExtInf::with_title(Duration::from_secs(5), M3u8String::new("foo").unwrap());
        assert_eq!("#EXTINF:5,foo".parse().ok(), Some(tag.clone()));
        assert_eq!(tag.to_string(), "#EXTINF:5,foo");
        assert_eq!(tag.requires_version(), ProtocolVersion::V1);

        let tag = ExtInf::new(Duration::from_millis(1234));
        assert_eq!("#EXTINF:1.234".parse().ok(), Some(tag.clone()));
        assert_eq!(tag.to_string(), "#EXTINF:1.234");
        assert_eq!(tag.requires_version(), ProtocolVersion::V3);
    }

    #[test]
    fn ext_x_byterange() {
        let tag = ExtXByteRange::new(ByteRange {
            length: 3,
            start: None,
        });
        assert_eq!("#EXT-X-BYTERANGE:3".parse().ok(), Some(tag));
        assert_eq!(tag.to_string(), "#EXT-X-BYTERANGE:3");
        assert_eq!(tag.requires_version(), ProtocolVersion::V4);

        let tag = ExtXByteRange::new(ByteRange {
            length: 3,
            start: Some(5),
        });
        assert_eq!("#EXT-X-BYTERANGE:3@5".parse().ok(), Some(tag));
        assert_eq!(tag.to_string(), "#EXT-X-BYTERANGE:3@5");
        assert_eq!(tag.requires_version(), ProtocolVersion::V4);
    }

    #[test]
    fn ext_x_discontinuity() {
        let tag = ExtXDiscontinuity;
        assert_eq!("#EXT-X-DISCONTINUITY".parse().ok(), Some(tag));
        assert_eq!(tag.to_string(), "#EXT-X-DISCONTINUITY");
        assert_eq!(tag.requires_version(), ProtocolVersion::V1);
    }

    #[test]
    fn ext_x_key() {
        let tag = ExtXKey::new_without_key();
        let text = "#EXT-X-KEY:METHOD=NONE";
        assert_eq!(text.parse().ok(), Some(tag.clone()));
        assert_eq!(tag.to_string(), text);
        assert_eq!(tag.requires_version(), ProtocolVersion::V1);

        let tag = ExtXKey::new(DecryptionKey {
            method: EncryptionMethod::Aes128,
            uri: QuotedString::new("foo").unwrap(),
            iv: None,
            key_format: None,
            key_format_versions: None,
        });
        let text = r#"#EXT-X-KEY:METHOD=AES-128,URI="foo""#;
        assert_eq!(text.parse().ok(), Some(tag.clone()));
        assert_eq!(tag.to_string(), text);
        assert_eq!(tag.requires_version(), ProtocolVersion::V1);

        let tag = ExtXKey::new(DecryptionKey {
            method: EncryptionMethod::Aes128,
            uri: QuotedString::new("foo").unwrap(),
            iv: Some(HexadecimalSequence::new(vec![0, 1, 2])),
            key_format: None,
            key_format_versions: None,
        });
        let text = r#"#EXT-X-KEY:METHOD=AES-128,URI="foo",IV=0x000102"#;
        assert_eq!(text.parse().ok(), Some(tag.clone()));
        assert_eq!(tag.to_string(), text);
        assert_eq!(tag.requires_version(), ProtocolVersion::V2);

        let tag = ExtXKey::new(DecryptionKey {
            method: EncryptionMethod::Aes128,
            uri: QuotedString::new("foo").unwrap(),
            iv: Some(HexadecimalSequence::new(vec![0, 1, 2])),
            key_format: Some(QuotedString::new("baz").unwrap()),
            key_format_versions: None,
        });
        let text = r#"#EXT-X-KEY:METHOD=AES-128,URI="foo",IV=0x000102,KEYFORMAT="baz""#;
        assert_eq!(text.parse().ok(), Some(tag.clone()));
        assert_eq!(tag.to_string(), text);
        assert_eq!(tag.requires_version(), ProtocolVersion::V5);
    }

    #[test]
    fn ext_x_map() {
        let tag = ExtXMap::new(QuotedString::new("foo").unwrap());
        let text = r#"#EXT-X-MAP:URI="foo""#;
        assert_eq!(text.parse().ok(), Some(tag.clone()));
        assert_eq!(tag.to_string(), text);
        assert_eq!(tag.requires_version(), ProtocolVersion::V6);

        let tag = ExtXMap::with_range(
            QuotedString::new("foo").unwrap(),
            ByteRange {
                length: 9,
                start: Some(2),
            },
        );
        let text = r#"#EXT-X-MAP:URI="foo",BYTERANGE=9@2"#;
        assert_eq!(text.parse().ok(), Some(tag.clone()));
        assert_eq!(tag.to_string(), text);
        assert_eq!(tag.requires_version(), ProtocolVersion::V6);
    }

    #[test]
    fn ext_x_program_date_time() {
        let text = "#EXT-X-PROGRAM-DATE-TIME:2010-02-19T14:54:23.031+08:00";
        assert!(text.parse::<ExtXProgramDateTime>().is_ok());

        let tag = text.parse::<ExtXProgramDateTime>().unwrap();
        assert_eq!(tag.to_string(), text);
        assert_eq!(tag.requires_version(), ProtocolVersion::V1);
    }
}
