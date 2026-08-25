// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
description: Empty or a function object may be used as options
features: [Temporal]
---*/

const instance = new Temporal.ZonedDateTime(0n, "UTC");

const result1 = instance.with({ day: 5 }, {});
assert.sameValue(
  result1.epochNanoseconds, 345600000000000n, "UTC",
  "options may be an empty plain object"
);

const result2 = instance.with({ day: 5 }, () => {});
assert.sameValue(
  result2.epochNanoseconds, 345600000000000n, "UTC",
  "options may be a function object"
);
