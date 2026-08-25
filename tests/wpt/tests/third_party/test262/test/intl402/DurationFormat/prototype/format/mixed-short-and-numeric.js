// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat.prototype.format
description: Checks that durations containing a mixture of numericlike and non-numericlike styles are formatted using the "short" style when DurationFormat base style is *undefined*.
info: |
    PartitionDurationFormatPattern ( durationFormat, duration )

      11. Perform ! CreateDataPropertyOrThrow(lfOpts, "type", "unit").
      12. Let listStyle be durationFormat.[[Style]].
      (...)
      14. Perform ! CreateDataPropertyOrThrow(lfOpts, "style", listStyle).

locale: [en]
features: [Intl.DurationFormat]
---*/

const locale = "en";
const timeSeparator = ":";


let d = {days: 5, hours: 1, minutes: 2, seconds: 3};
let dfOpts = {minutes: "numeric", seconds: "numeric"};

let expectedList = [];
expectedList.push(new Intl.NumberFormat(locale, {style: "unit", unit: "day", unitDisplay: "short"}).format(d.days));
expectedList.push(new Intl.NumberFormat(locale, {style: "unit", unit: "hour", unitDisplay: "short"}).format(d.hours));
expectedList.push(new Intl.NumberFormat(locale).format(d.minutes) + timeSeparator + new Intl.NumberFormat(locale, {minimumIntegerDigits: 2}).format(d.seconds));

let expected = new Intl.ListFormat(locale, {type: "unit", style: "short"}).format(expectedList);
let actual = new Intl.DurationFormat(locale, dfOpts).format(d);

assert.sameValue(actual, expected);

