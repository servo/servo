// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.add
description: Adding a duration object works.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// "1969-12-25T12:23:45.678901234+00:00[UTC]"
const zdt = new Temporal.ZonedDateTime(-560174321098766n, "UTC")
const d = new Temporal.Duration(0, 0, 0, 0, 240, 0, 0, 0, 0, 800);
// "1970-01-04T12:23:45.678902034+00:00[UTC]"
const expected = new Temporal.ZonedDateTime(303825678902034n, "UTC");

TemporalHelpers.assertZonedDateTimesEqual(zdt.add(d), expected);
