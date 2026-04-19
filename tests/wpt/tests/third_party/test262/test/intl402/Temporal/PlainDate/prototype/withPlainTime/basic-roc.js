// Copyright 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.withplaintime
description: withPlainTime should work for roc calendar dates
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

let timeRecord = {
  hour: 9,
  minute: 8,
  second: 7,
  millisecond: 6,
  microsecond: 5,
  nanosecond: 4
};

let d = new Temporal.PlainDateTime(2020, 3, 15, 4, 5, 6, 7, 8, 9, 'roc');
TemporalHelpers.assertPlainDateTime(d.withPlainTime(timeRecord), 109, 3, 'M03', 15, 9, 8, 7, 6, 5, 4, '', 'roc', 109);
