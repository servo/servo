// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: >
  ISO strings cannot have sub-minute UTC offset as a time zone identifier
features: [Temporal]
---*/

const str = "2021-08-19T17:30:45.123456789-12:12:59.9[-12:12:59.9]";
assert.throws(RangeError, () => Temporal.ZonedDateTime.from(str), `${str} is not a valid ISO string`);
