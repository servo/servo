// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime
description: ZonedDateTime constructor with non-integer arguments.
features: [Temporal]
---*/

assert.sameValue(new Temporal.ZonedDateTime(false, "UTC").epochNanoseconds,
  0n, "boolean defaults");

assert.sameValue(new Temporal.ZonedDateTime(true, "UTC").epochNanoseconds,
  1n, "boolean defaults");

assert.throws(TypeError, () => new Temporal.ZonedDateTime(Symbol(), "UTC"), `symbol`);
assert.throws(TypeError, () => new Temporal.ZonedDateTime(undefined, "UTC"), `undefined`);

