// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tostring
description: The time zone offset part of the string serialization (IANA time zones)
features: [BigInt, Temporal]
---*/

const instant = new Temporal.Instant(0n);

function test(timeZone, expected, description) {
  assert.sameValue(instant.toString({ timeZone }), expected, description);
}

test("Europe/Berlin", "1970-01-01T01:00:00+01:00", "positive offset");
test("America/New_York", "1969-12-31T19:00:00-05:00", "negative offset");
test("Africa/Monrovia", "1969-12-31T23:15:30-00:45", "sub-minute offset");
