// Copyright (C) 2023 Justin Grant. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.equals
description: The time zone identifier is case-insensitive
features: [Temporal]
---*/

const instance = new Temporal.ZonedDateTime(0n, "UTC");

const bag = { year: 1970, monthCode: "M01", day: 1, timeZone: "utC" };
const result1 = instance.equals(bag);
assert.sameValue(result1, true, "Time zone is case-insensitive with property bag argument");

const str = "1970-01-01[UtC]";
const result2 = instance.equals(str);
assert.sameValue(result2, true, "Time zone is case-insensitive with string argument");
