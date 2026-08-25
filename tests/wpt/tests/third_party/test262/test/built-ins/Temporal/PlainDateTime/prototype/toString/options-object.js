// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tostring
description: Empty or a function object may be used as options
features: [Temporal]
---*/

const instance = new Temporal.PlainDateTime(2000, 5, 2);

const result1 = instance.toString({});
assert.sameValue(
  result1, "2000-05-02T00:00:00",
  "options may be an empty plain object"
);

const result2 = instance.toString(() => {});
assert.sameValue(
  result2, "2000-05-02T00:00:00",
  "options may be a function object"
);
