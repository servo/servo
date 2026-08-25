// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.until
description: A variety of invalid option arguments
features: [Temporal, Symbol]
---*/

const feb20 = new Temporal.PlainDateTime(2020, 2, 1, 0, 0);
const feb21 = new Temporal.PlainDateTime(2021, 2, 1, 0, 0);

const badOptions = [null, 1, 'obviously invalid', true, Symbol('foo'), 1n];
badOptions.forEach((bad) => {
  assert.throws(
    TypeError,
    () => feb20.until(feb21, bad),
    `unacceptable options (${typeof bad})`
  );
});
