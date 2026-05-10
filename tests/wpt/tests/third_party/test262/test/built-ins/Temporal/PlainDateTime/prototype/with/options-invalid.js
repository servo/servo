// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.with
description: Verify that undefined options are handled correctly.
features: [Temporal, Symbol]
---*/

const datetime = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 456, 789);

const badOptions = [null, 1, 'hello', true, Symbol('foo'), 1n];

badOptions.forEach((bad) => {
  assert.throws(
    TypeError,
    () => datetime.with({ day: 5 }, bad),
    `bad options (${typeof bad})`
  );
});
