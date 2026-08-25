// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tozoneddatetime
description: Empty or a function object may be used as options
features: [Temporal]
---*/

const instance = new Temporal.PlainDateTime(2000, 5, 2);

const result1 = instance.toZonedDateTime("UTC", {});
assert.sameValue(
  result1.epochNanoseconds, 957225600000000000n,
  "options may be an empty plain object"
);

const result2 = instance.toZonedDateTime("UTC", () => {});
assert.sameValue(
  result2.epochNanoseconds, 957225600000000000n,
  "options may be a function object"
);
