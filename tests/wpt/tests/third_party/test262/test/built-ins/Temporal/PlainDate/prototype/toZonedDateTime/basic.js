// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.tozoneddatetime
description: Basic tests for toZonedDateTime().
features: [Temporal]
---*/

const plainDate = Temporal.PlainDate.from("2020-01-01");
const timeZone = "UTC";
const plainTime = Temporal.PlainTime.from("12:00");

let result = plainDate.toZonedDateTime({ timeZone, plainTime });
assert.sameValue(result.toString(), "2020-01-01T12:00:00+00:00[UTC]", "objects passed");

result = plainDate.toZonedDateTime(timeZone);
assert.sameValue(result.toString(), "2020-01-01T00:00:00+00:00[UTC]", "time zone object argument");

result = plainDate.toZonedDateTime({ timeZone, plainTime });
assert.sameValue(result.toString(), "2020-01-01T12:00:00+00:00[UTC]", "time zone string property");

result = plainDate.toZonedDateTime({ timeZone, plainTime: "12:00" });
assert.sameValue(result.toString(), "2020-01-01T12:00:00+00:00[UTC]", "time string property");
