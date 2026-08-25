// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.RelativeTimeFormat.prototype.format
description: Checks the handling of plural unit arguments to Intl.RelativeTimeFormat.prototype.format().
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

assert.sameValue(typeof rtf.format, "function", "format should be supported");

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
  assert.sameValue(rtf.format(3, unit + "s"), rtf.format(3, unit),
                   `Should support unit ${unit}s as a synonym for ${unit}`)
}
