// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.add
description: Math order of operations - add / reject.
features: [Temporal]
---*/

// const zdt = Temporal.ZonedDateTime.from("2020-01-31T00:00-08:00[-08:00]");
const zdt = new Temporal.ZonedDateTime(1580457600000000000n, "-08:00");
const d = new Temporal.Duration(0, 1, 0, 1, 0, 0, 0, 0, 0, 0);

const options = { overflow: "reject" };

assert.throws(RangeError, () => zdt.add(d, options));
