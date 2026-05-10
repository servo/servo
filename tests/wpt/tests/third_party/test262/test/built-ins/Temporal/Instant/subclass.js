// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant
description: Test for Temporal.Instant subclassing.
features: [Temporal]
---*/

class CustomInstant extends Temporal.Instant {
}

const instance = new CustomInstant(0n);
assert.sameValue(instance.epochNanoseconds, 0n);
assert.sameValue(Object.getPrototypeOf(instance), CustomInstant.prototype, "Instance of CustomInstant");
assert(instance instanceof CustomInstant, "Instance of CustomInstant");
assert(instance instanceof Temporal.Instant, "Instance of Temporal.Instant");
