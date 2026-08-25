// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-temporal.zoneddatetime.prototype.offset
description: Basic tests for Temporal.ZonedDateTime.prototype.offset
features: [BigInt, Temporal]
---*/

function test(timeZoneIdentifier, expectedOffsetString, description) {
  const datetime = new Temporal.ZonedDateTime(0n, timeZoneIdentifier);
  assert.sameValue(datetime.offset, expectedOffsetString, description);
}

test("UTC", "+00:00", "offset of UTC is +00:00");
test("+01:00", "+01:00", "positive offset");
test("-05:00", "-05:00", "negative offset");
