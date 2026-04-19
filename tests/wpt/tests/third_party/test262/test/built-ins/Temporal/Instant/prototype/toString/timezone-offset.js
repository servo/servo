// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tostring
description: The time zone offset part of the string serialization
features: [BigInt, Temporal]
---*/

const instant = new Temporal.Instant(0n);

function test(timeZoneIdentifier, expected, description) {
  assert.sameValue(instant.toString({ timeZone: timeZoneIdentifier }), expected, description);
}

test("UTC", "1970-01-01T00:00:00+00:00", "offset of UTC is +00:00");
test("+01:00", "1970-01-01T01:00:00+01:00", "positive offset");
test("-05:00", "1969-12-31T19:00:00-05:00", "negative offset");
