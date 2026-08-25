// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant
description: TypeError thrown if input is of wrong primitive type
features: [Symbol, Temporal]
---*/

assert.throws(TypeError, () => new Temporal.Instant(), "undefined");
assert.throws(TypeError, () => new Temporal.Instant(undefined), "undefined");
assert.throws(TypeError, () => new Temporal.Instant(null), "null");
assert.throws(TypeError, () => new Temporal.Instant(42), "number");
assert.throws(TypeError, () => new Temporal.Instant(Symbol()), "symbol");
