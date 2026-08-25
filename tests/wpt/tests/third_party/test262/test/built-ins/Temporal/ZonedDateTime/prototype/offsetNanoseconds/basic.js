// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.zoneddatetime.prototype.offsetnanoseconds
description: Basic tests for Temporal.ZonedDateTime.prototype.offsetNanoseconds
features: [BigInt, Temporal]
---*/

function test(timeZoneIdentifier, expectedOffsetNs, description) {
  const datetime = new Temporal.ZonedDateTime(0n, timeZoneIdentifier);
  assert.sameValue(datetime.offsetNanoseconds, expectedOffsetNs, description);
}

test("UTC", 0, "offset of UTC is +00:00");
test("+01:00", 3600e9, "positive offset");
test("-05:00", -5 * 3600e9, "negative offset");
