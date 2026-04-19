// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.compare
description: A calendar ID is valid input for Calendar
features: [Temporal]
---*/

const calendar = "iso8601";

const timeZone = "UTC";
const datetime = new Temporal.ZonedDateTime(0n, timeZone);
const arg = { year: 1970, monthCode: "M01", day: 1, timeZone, calendar };

const result1 = Temporal.ZonedDateTime.compare(arg, datetime);
assert.sameValue(result1, 0, `Calendar created from string "${arg}" (first argument)`);

const result2 = Temporal.ZonedDateTime.compare(datetime, arg);
assert.sameValue(result2, 0, `Calendar created from string "${arg}" (second argument)`);
