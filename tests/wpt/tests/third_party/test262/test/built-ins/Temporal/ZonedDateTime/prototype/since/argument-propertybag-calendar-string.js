// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.since
description: A calendar ID is valid input for Calendar
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const timeZone = "UTC";
const instance = new Temporal.ZonedDateTime(0n, timeZone);

const calendar = "iso8601";

const arg = { year: 1970, monthCode: "M01", day: 1, timeZone, calendar };
const result = instance.since(arg);
TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, `Calendar created from string "${calendar}"`);
