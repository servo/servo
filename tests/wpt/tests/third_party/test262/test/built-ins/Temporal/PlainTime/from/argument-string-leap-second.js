// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.from
description: Leap second is replaced by :59 in ISO strings.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

for (const options of [undefined, {}, { overflow: "constrain" }, { overflow: "reject" }]) {
  TemporalHelpers.assertPlainTime(Temporal.PlainTime.from("23:59:60", options),
    23, 59, 59, 0, 0, 0);
  TemporalHelpers.assertPlainTime(Temporal.PlainTime.from("12:30:60", options),
    12, 30, 59, 0, 0, 0);
  TemporalHelpers.assertPlainTime(Temporal.PlainTime.from("23:59:60.170", options),
    23, 59, 59, 170, 0, 0);
}
