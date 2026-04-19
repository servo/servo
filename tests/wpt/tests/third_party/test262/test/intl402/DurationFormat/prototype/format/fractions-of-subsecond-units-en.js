// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat.prototype.format
description: Checks that fractional milliseconds and microseconds are formatted correctly when microseconds or nanoseconds are the first units with "numeric" style.
info: |
    PartitionDurationFormatPattern ( durationFormat, duration )
    (...)
    9.
      (...)
      g. If unit is "seconds", "milliseconds", or "microseconds", then
        i. If unit is "seconds", then
            1. Let nextStyle be durationFormat.[[MillisecondsStyle]].
        ii. Else if unit is "milliseconds", then
            1. Let nextStyle be durationFormat.[[MicrosecondsStyle]].
        iii. Else,
            1. Let nextStyle be durationFormat.[[NanosecondsStyle]].
        iv. If nextStyle is "fractional", then
            1. Set value to value + AddFractionalDigits(durationFormat, duration).
            2. If durationFormat.[[FractionalDigits]] is undefined, then
                a. Let maximumFractionDigits be 9𝔽.
                b. Let minimumFractionDigits be +0𝔽.
            3. Else,
                a. Let maximumFractionDigits be 𝔽(durationFormat.[[FractionalDigits]]).
                b. Let minimumFractionDigits be 𝔽(durationFormat.[[FractionalDigits]]).
            4. Perform ! CreateDataPropertyOrThrow(nfOpts, "maximumFractionDigits", maximumFractionDigits ).
            5. Perform ! CreateDataPropertyOrThrow(nfOpts, "minimumFractionDigits", minimumFractionDigits ).
            6. Perform ! CreateDataPropertyOrThrow(nfOpts, "roundingMode", "trunc").
            7. Set done to true.

locale: [en]
features: [Intl.DurationFormat]
---*/


const locale = "en";
const decimalSeparator = ".";

let d = {seconds: 3, milliseconds: 444, microseconds: 55, nanoseconds: 6};
let dfOpts = {microseconds: "numeric"};

let expectedList = [];
expectedList.push(new Intl.NumberFormat(locale, {style: "unit", unit: "second", unitDisplay: "short"}).format(d.seconds));
expectedList.push(new Intl.NumberFormat(locale, {style: "unit", unit: "millisecond", unitDisplay: "short", minimumFractionDigits:0, maximumFractionDigits: 9, roundingMode: "trunc"}).format(d.milliseconds.toString() + decimalSeparator + d.microseconds.toString().padStart(3, '0') + d.nanoseconds.toString().padStart(3, '0')));

let expected = new Intl.ListFormat(locale, {type: "unit", style: "short"}).format(expectedList);
let actual = new Intl.DurationFormat(locale, dfOpts).format(d);

assert.sameValue(actual, expected, `DurationFormat output when microseconds first "numeric" unit`);

dfOpts = {nanoseconds: "numeric"};
expectedList = [];

expectedList.push(new Intl.NumberFormat(locale, {style: "unit", unit: "second", unitDisplay: "short"}).format(d.seconds));
expectedList.push(new Intl.NumberFormat(locale, {style: "unit", unit: "millisecond", unitDisplay: "short"}).format(d.milliseconds));
expectedList.push(new Intl.NumberFormat(locale, {style: "unit", unit: "microsecond", unitDisplay: "short", minimumFractionDigits:0, maximumFractionDigits: 9, roundingMode: "trunc"}).format(d.microseconds.toString() + decimalSeparator + d.nanoseconds.toString().padStart(3, '0')));

expected = new Intl.ListFormat(locale, {type: "unit", style: "short"}).format(expectedList);
actual = new Intl.DurationFormat(locale, dfOpts).format(d);

assert.sameValue(actual, expected, `DurationFormat output when nanoseconds first "numeric" unit`);
