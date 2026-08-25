// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.equals
description: Leap second is a valid ISO string for a calendar in a property bag
features: [Temporal]
---*/

const timeZone = "UTC";
const instance = new Temporal.ZonedDateTime(0n, timeZone);

const calendar = "2016-12-31T23:59:60+00:00[UTC]";

const arg = { year: 1970, monthCode: "M01", day: 1, timeZone, calendar };
const result = instance.equals(arg);
assert.sameValue(
  result,
  true,
  "leap second is a valid ISO string for calendar"
);
