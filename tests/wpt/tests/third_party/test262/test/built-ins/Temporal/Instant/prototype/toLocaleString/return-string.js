// Copyright (C) 2021 Kate Miháliková. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tolocalestring
description: >
    toLocaleString return a string.
features: [Temporal]
---*/

const instant = new Temporal.Instant(957270896_987_650_000n);

assert.sameValue(typeof instant.toLocaleString("en", { dateStyle: "short" }), "string");
assert.sameValue(typeof instant.toLocaleString("en", { timeStyle: "short" }), "string");
