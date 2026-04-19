// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.withplaintime
description: A plain object can be used as an argument
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const dt = new Temporal.PlainDateTime(2015, 12, 7, 3, 24, 30, 0, 3, 500);

assert.throws(
  TypeError,
  () => dt.withPlainTime({}),
  "empty object not an acceptable argument"
);

TemporalHelpers.assertPlainDateTime(
  dt.withPlainTime({ hour: 10 }),
  2015, 12, "M12", 7, 10, 0, 0, 0, 0, 0,
  "plain object (hour) works"
);

assert.throws(
  TypeError,
  () => dt.withPlainTime({ hours: 9 }), // should be "hour", see above
  "plain object with a single unrecognized property fails"
);

TemporalHelpers.assertPlainDateTime(
  dt.withPlainTime({ hour: 10, seconds: 123 }),
  2015, 12, "M12", 7, 10, 0, 0, 0, 0, 0,
  "unrecognized properties are ignored if at least one recognized property is present"
);
