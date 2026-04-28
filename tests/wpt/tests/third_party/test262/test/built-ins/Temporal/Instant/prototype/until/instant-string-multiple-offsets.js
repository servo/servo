// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.until
description: Instant strings with UTC offset fractional part are not confused with time fractional part
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.Instant(0n);
const str = "1970-01-01T00:02:00.000000000+00:02[+01:30]";

const result = instance.until(str);
TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, "UTC offset determined from offset part of string");
