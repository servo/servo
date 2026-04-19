// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime
description: Test for Temporal.ZonedDateTime subclassing.
features: [Temporal]
---*/

class CustomZonedDateTime extends Temporal.ZonedDateTime {
}

const instance = new CustomZonedDateTime(0n, "UTC");
assert.sameValue(instance.epochNanoseconds, 0n);
assert.sameValue(Object.getPrototypeOf(instance), CustomZonedDateTime.prototype, "Instance of CustomZonedDateTime");
assert(instance instanceof CustomZonedDateTime, "Instance of CustomZonedDateTime");
assert(instance instanceof Temporal.ZonedDateTime, "Instance of Temporal.ZonedDateTime");
