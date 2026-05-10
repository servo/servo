// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Throws if a string argument is supplied
esid: sec-temporal.plaindatetime.prototype.with
features: [Temporal]
---*/

const instance = new Temporal.PlainDateTime(2000, 5, 2, 12, 34, 56, 987, 654, 321);

const baddies = ["12:00", "1995-04-07", "2019-05-17T12:34:56.007007007", "2019-05-17T12:34:56.007007007Z", "42"];

baddies.forEach((bad) => {
  assert.throws(
    TypeError,
    () => instance.with(bad),
    `bad argument (${bad})`
  );
});
