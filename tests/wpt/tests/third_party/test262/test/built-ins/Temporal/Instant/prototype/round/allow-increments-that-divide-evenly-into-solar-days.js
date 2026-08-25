// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.round
description: round() allows increments that divide evenly into solar days.
features: [Temporal]
---*/

const inst = Temporal.Instant.from("1976-11-18T14:23:30.123456789Z");

assert(inst.round({
    smallestUnit: "second",
    roundingIncrement: 864
}) instanceof Temporal.Instant);
