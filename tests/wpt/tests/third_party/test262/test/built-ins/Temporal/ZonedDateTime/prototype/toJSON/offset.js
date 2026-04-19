// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.tojson
description: The time zone offset part of the string serialization
features: [BigInt, Temporal]
---*/

function test(timeZoneIdentifier, expected, description) {
  const datetime = new Temporal.ZonedDateTime(0n, timeZoneIdentifier);
  assert.sameValue(datetime.toJSON(), expected, description);
}

test("UTC", "1970-01-01T00:00:00+00:00[UTC]", "offset of UTC is +00:00");
test("+01:00", "1970-01-01T01:00:00+01:00[+01:00]", "positive offset");
test("-05:00", "1969-12-31T19:00:00-05:00[-05:00]", "negative offset");
