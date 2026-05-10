// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tozoneddatetimeiso
description: toZonedDateTimeISO() throws without parameter.
features: [Temporal]
---*/

const inst = Temporal.Instant.from("1976-11-18T14:23:30.123456789Z");

assert.throws(TypeError, () => inst.toZonedDateTimeISO());
