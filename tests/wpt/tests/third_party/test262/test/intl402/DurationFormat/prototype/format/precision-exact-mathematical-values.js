// Copyright (C) 2023 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat.prototype.format
description: >
  PartitionDurationFormatPattern computes on exact mathematical values.
info: |
  PartitionDurationFormatPattern ( durationFormat, duration )
  ...
  4. While done is false, repeat for each row in Table 1 in order, except the header row:
    ...
    j. If unit is "seconds", "milliseconds", or "microseconds", then
      i. If unit is "seconds", then
        1. Let nextStyle be durationFormat.[[MillisecondsStyle]].
      ...
      iv. If nextStyle is "numeric", then
        1. If unit is "seconds", then
          a. Set value to value + duration.[[Milliseconds]] / 10^3 + duration.[[Microseconds]] / 10^6 + duration.[[Nanoseconds]] / 10^9.
    ...
    l. If value is not 0 or display is not "auto", then
      ii. If style is "2-digit" or "numeric", then
        ...
        7. Let parts be ! PartitionNumberPattern(nf, value).
        ...

locale: [en]
includes: [testIntl.js]
features: [Intl.DurationFormat]
---*/

const durations = [
  // 10000000 + (1 / 10^9)
  // = 10000000.000000001
  {
    seconds: 10_000_000,
    nanoseconds: 1,
  },

  // 1 + (2 / 10^3) + (3 / 10^6) + (9007199254740991 / 10^9)
  // = 9.007200256743991 × 10^6
  {
    seconds: 1,
    milliseconds: 2,
    microseconds: 3,
    nanoseconds: Number.MAX_SAFE_INTEGER,
  },

  // (4503599627370497024 / 10^3) + (4503599627370494951424 / 10^6)
  // = 4503599627370497.024 + 4503599627370494.951424
  // = 9007199254740991.975424
  {
    // Actual value is: 4503599627370497024
    milliseconds: 4503599627370497_000,

    // Actual value is: 4503599627370494951424
    microseconds: 4503599627370495_000000,
  },
];

const df = new Intl.DurationFormat("en", {style: "digital"});

for (let duration of durations) {
  let expected = formatDurationFormatPattern(df, duration);
  assert.sameValue(
    df.format(duration),
    expected,
    `Duration is ${JSON.stringify(duration)}`
  );
}
