// Copyright 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
description: Basic tests with options
features: [Temporal]
---*/

[
  { overflow: 'constrain' },
  { overflow: 'reject' }
].forEach(function (validOptions) {
  let d = new Temporal.PlainDate(1, 2, 3);
  let d2 = Temporal.PlainDate.from(d, validOptions);
  assert.sameValue(d2.year, 1);
  assert.sameValue(d2.month, 2);
  assert.sameValue(d2.day, 3);
  assert.sameValue(d2.calendarId, 'iso8601');
});
