// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
description: RangeError thrown if a string with trailing junk is used as a PlainDate
features: [Temporal, arrow-function]
---*/

assert.throws(RangeError, () => Temporal.PlainDate.from("1976-11-18junk"),
  "String with trailing junk should not be valid as a PlainDate");
