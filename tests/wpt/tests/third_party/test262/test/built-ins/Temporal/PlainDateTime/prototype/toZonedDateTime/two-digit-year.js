// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tozoneddatetime
description: >
  Check result when Gregorian year is two digits (in case underlying
  implementation uses Date and forgets to use setFullYear)
features: [Temporal]
---*/

const instance = new Temporal.PlainDateTime(98, 10, 29, 10, 46, 38, 271, 986, 102);
["earlier", "later", "compatible", "reject"].forEach((disambiguation) => {
  const result = instance.toZonedDateTime("+06:00", { disambiguation });
  assert.sameValue(result.epochNanoseconds, -59048507601_728_013_898n, "Result is 0098-10-29T04:46:38.271986102Z regardless of disambiguation");
});
