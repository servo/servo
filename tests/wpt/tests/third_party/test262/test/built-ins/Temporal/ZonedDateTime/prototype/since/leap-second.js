// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.since
description: Leap second is a valid ISO string for ZonedDateTime
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const timeZone = "UTC";
const instance = new Temporal.ZonedDateTime(1_483_228_799_000_000_000n, timeZone);

let arg = "2016-12-31T23:59:60+00:00[UTC]";
const result = instance.since(arg);
TemporalHelpers.assertDuration(
  result,
  0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  "leap second is a valid ISO string for ZonedDateTime"
);

arg = "2000-05-02T12:34:56+23:59[+23:59:60]";
assert.throws(
  RangeError,
  () => instance.since(arg),
  "leap second in time zone name not valid"
);
