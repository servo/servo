// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tozoneddatetime
description: Basic tests for disambiguation option
features: [Temporal]
---*/

const dtm = new Temporal.PlainDateTime(2000, 10, 29, 1, 45);

for (const disambiguation of ["compatible", "earlier", "later", "reject"]) {
  const result = dtm.toZonedDateTime("UTC", { disambiguation });
  assert.sameValue(result.epochNanoseconds, 972783900_000_000_000n, "epoch nanoseconds remains constant");
  assert.sameValue(result.timeZoneId, "UTC", "time zone is adopted");
}

for (const disambiguation of ["compatible", "earlier", "later", "reject"]) {
  const result = dtm.toZonedDateTime("+03:30", { disambiguation });
  assert.sameValue(result.epochNanoseconds, 972771300_000_000_000n, "epoch nanoseconds remains constant");
  assert.sameValue(result.timeZoneId, "+03:30", "time zone is adopted");
}
