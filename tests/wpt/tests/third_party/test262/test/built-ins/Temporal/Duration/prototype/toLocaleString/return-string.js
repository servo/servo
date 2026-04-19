// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.tolocalestring
description: >
    toLocaleString return a string.
features: [Temporal]
---*/

var duration = new Temporal.Duration();

assert.sameValue(typeof duration.toLocaleString(), "string");
