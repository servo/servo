// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.subtract
description: Throws with overflow reject
features: [Temporal]
---*/

const mar31 = Temporal.PlainDate.from("2020-03-31");
assert.throws(RangeError, () => mar31.subtract({ months: 1 }, { overflow: "reject" }));
