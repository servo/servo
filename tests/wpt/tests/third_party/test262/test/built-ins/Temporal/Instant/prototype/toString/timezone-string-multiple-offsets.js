// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tostring
description: Time zone strings with UTC offset are not confused with time
features: [Temporal]
---*/

const instance = new Temporal.Instant(0n);
const timeZone = "2021-08-19T17:30:45.123456789-12:12[+01:46]";

const result = instance.toString({ timeZone });
assert.sameValue(result.slice(-6), "+01:46", "Time zone string determined from bracket name");
