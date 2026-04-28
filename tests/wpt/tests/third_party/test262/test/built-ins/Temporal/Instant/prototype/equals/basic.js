// Copyright 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.equals
description: Basic functionality for Temporal.Instant.prototype.equals.
features: [Temporal]
---*/

let inst1 = new Temporal.Instant(1234567890123456789n);
let inst2 = new Temporal.Instant(1234567890123456000n);
let inst3 = new Temporal.Instant(1234567890123456000n);
assert(!inst1.equals(inst2));
assert(inst2.equals(inst3));
