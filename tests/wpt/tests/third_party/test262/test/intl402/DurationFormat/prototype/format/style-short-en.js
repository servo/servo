// Copyright 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat.prototype.format
description: Test if format method formats duration correctly with different "style" arguments
locale: [en]
includes: [testIntl.js]
features: [Intl.DurationFormat]
---*/

const style = "short";

const duration = {
  years: 1,
  months: 2,
  weeks: 3,
  days: 3,
  hours: 4,
  minutes: 5,
  seconds: 6,
  milliseconds: 7,
  microseconds: 8,
  nanoseconds: 9,
};

const df = new Intl.DurationFormat("en", {style});

const expected = formatDurationFormatPattern(df, duration);

assert.sameValue(df.format(duration), expected, `Assert DurationFormat format output using ${style} style option`);
