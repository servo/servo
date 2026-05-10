// Copyright 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.with
description: Time units in the property bag are ignored
includes: [temporalHelpers.js]
features: [Temporal]
---*/

let d1 = new Temporal.PlainDate(1911, 10, 10);
TemporalHelpers.assertPlainDate(d1.with({
  year: 2021,
  hour: 30
}), 2021, 10, 'M10', 10);
TemporalHelpers.assertPlainDate(d1.with({
  month: 11,
  minute: 71
}), 1911, 11, 'M11', 10);
TemporalHelpers.assertPlainDate(d1.with({
  monthCode: 'M05',
  second: 90
}), 1911, 5, 'M05', 10);
TemporalHelpers.assertPlainDate(d1.with({
  day: 30,
  era: 'BC'
}), 1911, 10, 'M10', 30);
