// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.since
description: Verify that invalid options are handled correctly.
features: [Temporal]
---*/

const feb20 = new Temporal.PlainYearMonth(2020, 2);
const feb21 = new Temporal.PlainYearMonth(2021, 2);

[
  null,
  1,
  'hello',
  true,
  Symbol('foo'),
  1n
].forEach((badOption) =>
  assert.throws(TypeError, () => feb21.since(feb20, badOption), `${String(badOption)} throws TypeError`)
);
