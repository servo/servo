// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tozoneddatetime
description: >
  Check result in Gregorian year zero (in case underlying
  implementation uses Date and calls setFullYear on a non-leap year)
features: [Temporal]
---*/

const instance = new Temporal.PlainDateTime(0, 10, 29, 10, 46, 38, 271, 986, 102);
["earlier", "later", "compatible", "reject"].forEach((disambiguation) => {
  const result = instance.toZonedDateTime("+06:00", { disambiguation });
  assert.sameValue(result.epochNanoseconds, -62141109201_728_013_898n, "Result is 0000-10-29T04:46:38.271986102Z regardless of disambiguation");
});

// leap day
const instanceLeap = new Temporal.PlainDateTime(0, 2, 29);
["earlier", "later", "compatible", "reject"].forEach((disambiguation) => {
  const result = instanceLeap.toZonedDateTime("-00:01", { disambiguation });
  assert.sameValue(result.epochNanoseconds, -62162121540_000_000_000n, "Result is 0000-02-29T00:01Z regardless of disambiguation");
});
