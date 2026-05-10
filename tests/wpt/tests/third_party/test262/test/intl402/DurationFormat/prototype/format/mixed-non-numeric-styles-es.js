// Copyright 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat.prototype.format
description: Test if format method formats duration correctly with mixed non-numeric settings for unit styles. Compares output from format method to output produced through using NumberFormat and ListFormat as used by DurationFormat.
info: |
  PartitionDurationFormatPattern ( durationFormat, duration )
  ...
  9. While done is false, repeat for each row in Table 2 in table order, except the header row:
    ...
    k. If value is not 0 or display is not "auto" or displayRequired is "true", then
      ...
      v. If style is not "fractional", "numeric", or "2-digit", then
        1. Perform ! CreateDataPropertyOrThrow(nfOpts, "style", "unit").
        2. Perform ! CreateDataPropertyOrThrow(nfOpts, "unit", numberFormatUnit).
        3. Perform ! CreateDataPropertyOrThrow(nfOpts, "unitDisplay", style).
      ...
      ix. Let parts be ! PartitionNumberPattern(nf, value).
  ...
  14. Perform ! CreateDataPropertyOrThrow(lfOpts, "style", listStyle).
  15. Let lf be ! Construct(%ListFormat%, « durationFormat.[[Locale]], lfOpts »).
  ...
  18. Let formatted be CreatePartsFromList(lf, strings).

locale: [es]
features: [Intl.DurationFormat]
---*/

function formatDuration(locale, duration, dfOpts){
  let result = [];
  for (const unit in duration){
    let nfUnit = unit.substring(0, unit.length - 1);
    let nf = new Intl.NumberFormat(locale, {style: "unit", unit: nfUnit, unitDisplay: dfOpts[unit]});
    result.push(nf.format(duration[unit]));
  }

  for (const baseStyle of ["long", "short", "narrow"]){
    let lf = new Intl.ListFormat(locale, {type: "unit", style: baseStyle});
    let expected = lf.format(result);
    dfOpts.style = baseStyle;
    let df = new Intl.DurationFormat(locale, dfOpts);
    let actual = df.format(duration);
    assert.sameValue(actual, expected);
  }
}

const duration = {
  years: 1,
  months: 2,
  weeks: 3,
  days: 0,
  hours: 4,
  minutes: 5,
  seconds: 6,
  milliseconds: 7,
  microseconds: 8,
  nanoseconds: 9,
};

const locale = "es";

formatDuration(locale, duration, {
  years: "narrow",
  months: "narrow",
  weeks: "narrow",
  days: "short",
  hours: "short",
  minutes: "short",
  seconds: "long",
  milliseconds: "long",
  microseconds: "long",
  nanoseconds: "narrow", });

formatDuration(locale, duration, {
  years: "long",
  months: "short",
  weeks: "narrow",
  days: "long",
  hours: "short",
  minutes: "narrow",
  seconds: "long",
  milliseconds: "short",
  microseconds: "narrow",
  nanoseconds: "long", });
