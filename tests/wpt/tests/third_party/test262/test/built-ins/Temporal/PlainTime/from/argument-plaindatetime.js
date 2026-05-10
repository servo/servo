// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.from
description: Fast path for converting Temporal.PlainDateTime to Temporal.PlainTime by reading internal slots
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

TemporalHelpers.checkPlainDateTimeConversionFastPath((plainDateTime) => {
  const result = Temporal.PlainTime.from(plainDateTime);
  TemporalHelpers.assertPlainTime(result, 12, 34, 56, 987, 654, 321);
});
