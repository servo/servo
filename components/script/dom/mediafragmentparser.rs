/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::borrow::Cow;
use std::collections::VecDeque;
use std::str::FromStr;

use chrono::NaiveDateTime;
use servo_url::ServoUrl;
use url::{form_urlencoded, Position, Url};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpatialRegion {
    Pixel,
    Percent,
}

impl FromStr for SpatialRegion {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pixel" => Ok(SpatialRegion::Pixel),
            "percent" => Ok(SpatialRegion::Percent),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpatialClipping {
    region: Option<SpatialRegion>,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MediaFragmentParser {
    id: Option<String>,
    tracks: Vec<String>,
    spatial: Option<SpatialClipping>,
    start: Option<f64>,
    end: Option<f64>,
}

impl MediaFragmentParser {
    pub fn id(&self) -> Option<String> {
        self.id.clone()
    }

    pub fn tracks(&self) -> &Vec<String> {
        self.tracks.as_ref()
    }

    pub fn start(&self) -> Option<f64> {
        self.start
    }

    // Parse an str of key value pairs, a URL, or a fragment.
    pub fn parse(input: &str) -> MediaFragmentParser {
        let mut parser = MediaFragmentParser::default();
        let (query, fragment) = split_url(input);
        let mut octets = decode_octets(query.as_bytes());
        octets.extend(decode_octets(fragment.as_bytes()));

        if !octets.is_empty() {
            for (key, value) in octets.iter() {
                match key.as_bytes() {
                    b"t" => {
                        if let Ok((start, end)) = parser.parse_temporal(value) {
                            parser.start = start;
                            parser.end = end;
                        }
                    },
                    b"xywh" => {
                        if let Ok(spatial) = parser.parse_spatial(value) {
                            parser.spatial = Some(spatial);
                        }
                    },
                    b"id" => parser.id = Some(value.to_string()),
                    b"track" => parser.tracks.push(value.to_string()),
                    _ => {},
                }
            }
            parser
        } else {
            if let Ok((start, end)) = parser.parse_temporal(input) {
                parser.start = start;
                parser.end = end;
            } else if let Ok(spatial) = parser.parse_spatial(input) {
                parser.spatial = Some(spatial);
            }
            parser
        }
    }

    // Either NPT or UTC timestamp (real world clock time).
    fn parse_temporal(&self, input: &str) -> Result<(Option<f64>, Option<f64>), ()> {
        let (_, fragment) = split_prefix(input);

        if fragment.ends_with('Z') || fragment.ends_with("Z-") {
            return self.parse_utc_timestamp(fragment);
        }

        if fragment.starts_with(',') || !fragment.contains(',') {
            let sec = parse_hms(&fragment.replace(',', ""))?;
            if fragment.starts_with(',') {
                Ok((Some(0.), Some(sec)))
            } else {
                Ok((Some(sec), None))
            }
        } else {
            let mut iterator = fragment.split(',');
            let start = parse_hms(iterator.next().ok_or(())?)?;
            let end = parse_hms(iterator.next().ok_or(())?)?;

            if iterator.next().is_some() || start >= end {
                return Err(());
            }

            Ok((Some(start), Some(end)))
        }
    }

    fn parse_utc_timestamp(&self, input: &str) -> Result<(Option<f64>, Option<f64>), ()> {
        if input.ends_with('-') || input.starts_with(',') || !input.contains('-') {
            let sec = parse_hms(
                NaiveDateTime::parse_from_str(&input.replace(['-', ','], ""), "%Y%m%dT%H%M%S%.fZ")
                    .map_err(|_| ())?
                    .time()
                    .to_string()
                    .as_ref(),
            )?;
            if input.starts_with(',') {
                Ok((Some(0.), Some(sec)))
            } else {
                Ok((Some(sec), None))
            }
        } else {
            let vec: Vec<&str> = input.split('-').collect();
            let mut hms: Vec<f64> = vec
                .iter()
                .flat_map(|s| NaiveDateTime::parse_from_str(s, "%Y%m%dT%H%M%S%.fZ"))
                .flat_map(|s| parse_hms(&s.time().to_string()))
                .collect();

            let end = hms.pop().ok_or(())?;
            let start = hms.pop().ok_or(())?;

            if !hms.is_empty() || start >= end {
                return Err(());
            }

            Ok((Some(start), Some(end)))
        }
    }

    fn parse_spatial(&self, input: &str) -> Result<SpatialClipping, ()> {
        let (prefix, s) = split_prefix(input);
        let vec: Vec<&str> = s.split(',').collect();
        let mut queue: VecDeque<u32> = vec.iter().flat_map(|s| s.parse::<u32>()).collect();

        let mut clipping = SpatialClipping {
            region: None,
            x: queue.pop_front().ok_or(())?,
            y: queue.pop_front().ok_or(())?,
            width: queue.pop_front().ok_or(())?,
            height: queue.pop_front().ok_or(())?,
        };

        if !queue.is_empty() {
            return Err(());
        }

        if let Some(s) = prefix {
            let region = SpatialRegion::from_str(s)?;
            if region.eq(&SpatialRegion::Percent) &&
                (clipping.x + clipping.width > 100 || clipping.y + clipping.height > 100)
            {
                return Err(());
            }
            clipping.region = Some(region);
        }

        Ok(clipping)
    }
}

impl From<&Url> for MediaFragmentParser {
    fn from(url: &Url) -> Self {
        let input: &str = &url[Position::AfterPath..];
        MediaFragmentParser::parse(input)
    }
}

impl From<&ServoUrl> for MediaFragmentParser {
    fn from(servo_url: &ServoUrl) -> Self {
        let input: &str = &servo_url[Position::AfterPath..];
        MediaFragmentParser::parse(input)
    }
}

// 5.1.1 Processing name-value components.
fn decode_octets(bytes: &[u8]) -> Vec<(Cow<str>, Cow<str>)> {
    form_urlencoded::parse(bytes)
        .filter(|(key, _)| matches!(key.as_bytes(), b"t" | b"track" | b"id" | b"xywh"))
        .collect()
}

// Parse a full URL or a relative URL without a base retaining the query and/or fragment.
fn split_url(s: &str) -> (&str, &str) {
    if s.contains('?') || s.contains('#') {
        for (index, byte) in s.bytes().enumerate() {
            if byte == b'?' {
                let partial = &s[index + 1..];
                for (i, byte) in partial.bytes().enumerate() {
                    if byte == b'#' {
                        return (&partial[..i], &partial[i + 1..]);
                    }
                }
                return (partial, "");
            }

            if byte == b'#' {
                return ("", &s[index + 1..]);
            }
        }
    }
    ("", s)
}

fn is_byte_number(byte: u8) -> bool {
    matches!(byte, 48..=57)
}

fn split_prefix(s: &str) -> (Option<&str>, &str) {
    for (index, byte) in s.bytes().enumerate() {
        if index == 0 && is_byte_number(byte) {
            break;
        }

        if byte == b':' {
            return (Some(&s[..index]), &s[index + 1..]);
        }
    }
    (None, s)
}

fn hms_to_seconds(hour: u32, minutes: u32, seconds: f64) -> f64 {
    let mut sec: f64 = f64::from(hour) * 3600.;
    sec += f64::from(minutes) * 60.;
    sec += seconds;
    sec
}

fn parse_npt_minute(s: &str) -> Result<u32, ()> {
    if s.len() > 2 {
        return Err(());
    }

    let minute = s.parse().map_err(|_| ())?;
    if minute > 59 {
        return Err(());
    }

    Ok(minute)
}

fn parse_npt_seconds(s: &str) -> Result<f64, ()> {
    if s.contains('.') {
        let mut iterator = s.split('.');
        if let Some(s) = iterator.next() {
            if s.len() > 2 {
                return Err(());
            }
            let sec = s.parse::<u32>().map_err(|_| ())?;
            if sec > 59 {
                return Err(());
            }
        }

        let _ = iterator.next();
        if iterator.next().is_some() {
            return Err(());
        }
    }

    s.parse().map_err(|_| ())
}

fn parse_hms(s: &str) -> Result<f64, ()> {
    let mut vec: VecDeque<&str> = s.split(':').collect();
    vec.retain(|x| !x.eq(&""));

    let result = match vec.len() {
        1 => {
            let secs = vec.pop_front().ok_or(())?.parse::<f64>().map_err(|_| ())?;

            if secs == 0. {
                return Err(());
            }

            hms_to_seconds(0, 0, secs)
        },
        2 => hms_to_seconds(
            0,
            parse_npt_minute(vec.pop_front().ok_or(())?)?,
            parse_npt_seconds(vec.pop_front().ok_or(())?)?,
        ),
        3 => hms_to_seconds(
            vec.pop_front().ok_or(())?.parse().map_err(|_| ())?,
            parse_npt_minute(vec.pop_front().ok_or(())?)?,
            parse_npt_seconds(vec.pop_front().ok_or(())?)?,
        ),
        _ => return Err(()),
    };

    if !vec.is_empty() {
        return Err(());
    }

    Ok(result)
}
