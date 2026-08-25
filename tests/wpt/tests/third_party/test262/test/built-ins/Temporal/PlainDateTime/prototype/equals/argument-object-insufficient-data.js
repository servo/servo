// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.equals
description: If argument is an object, it must contain sufficient information
features: [Temporal]
---*/

const dt = new Temporal.PlainDateTime(2019, 10, 29, 10, 46, 38, 271, 986, 102);

assert.throws(
  TypeError,
  () => dt.equals({ year: 1976 }),
  "object must contain required properties"
);
