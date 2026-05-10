// Copyright (C) 2021 Kate Miháliková. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.tolocalestring
description: >
    toLocaleString return a string.
features: [Temporal]
---*/

const time = new Temporal.PlainTime(12, 34, 56, 987, 654, 321);

assert.sameValue(typeof time.toLocaleString("en", { timeStyle: "short" }), "string");
