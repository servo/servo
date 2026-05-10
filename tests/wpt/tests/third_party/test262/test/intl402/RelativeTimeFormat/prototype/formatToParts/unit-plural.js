// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.RelativeTimeFormat.prototype.formatToParts
description: Checks the handling of plural unit arguments to Intl.RelativeTimeFormat.prototype.formatToParts().
info: |
    SingularRelativeTimeUnit ( unit )

    2. If unit is "seconds", return "second".
    3. If unit is "minutes", return "minute".
    4. If unit is "hours", return "hour".
    5. If unit is "days", return "day".
    6. If unit is "weeks", return "week".
    7. If unit is "months", return "month".
    8. If unit is "quarters", return "quarter".
    9. If unit is "years", return "year".

features: [Intl.RelativeTimeFormat]
---*/

const rtf = new Intl.RelativeTimeFormat("en-US");

assert.sameValue(typeof rtf.formatToParts, "function", "formatToParts should be supported");

const units = [
  "second",
  "minute",
  "hour",
  "day",
  "week",
  "month",
  "quarter",
  "year",
];

for (const unit of units) {
  const plural = rtf.formatToParts(3, unit + "s");
  const singular = rtf.formatToParts(3, unit);

  assert.sameValue(plural.length, singular.length,
                   `Should support unit ${unit}s as a synonym for ${unit}: length`);

  for (let i = 0; i < plural.length; ++i) {
    assert.sameValue(plural[i].type, singular[i].type,
                     `Should support unit ${unit}s as a synonym for ${unit}: [${i}].type`);
    assert.sameValue(plural[i].value, singular[i].value,
                     `Should support unit ${unit}s as a synonym for ${unit}: [${i}].value`);
    assert.sameValue(plural[i].unit, singular[i].unit,
                     `Should support unit ${unit}s as a synonym for ${unit}: [${i}].unit`);
  }
}
