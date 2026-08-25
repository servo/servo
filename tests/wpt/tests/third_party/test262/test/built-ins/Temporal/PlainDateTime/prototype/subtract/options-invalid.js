// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.subtract
description: Various invalid (wrong type) values for options argument
features: [Temporal, Symbol]
---*/

const jan31 = new Temporal.PlainDateTime(2020, 1, 31, 15, 0);

const badOptions = [null, 1, 'hello', true, Symbol('foo'), 1n];

badOptions.forEach((bad) => {
  assert.throws(
    TypeError,
    () => jan31.subtract({ years: 1 }, bad),
    `invalid options (${typeof bad})`
  );
});
