// Copyright (C) 2023 Justin Grant. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.subtract
description: Subtraction resulting in a pre-1883 date returns the correct offset
features: [Temporal]
---*/

const instance = Temporal.ZonedDateTime.from("2001-09-08T18:46:40-07:00[America/Vancouver]");
const expectedOffset = "-08:12:28"; // Offset before introduction of standard time zones

assert.sameValue(instance.subtract(Temporal.Duration.from("P5432Y1837M")).offset, expectedOffset, `subtract -P5432Y1837M, offset should be ${expectedOffset}`);
assert.sameValue(instance.subtract(Temporal.Duration.from("P5432Y1836M")).offset, expectedOffset, `subtract -P5432Y1836M, offset should be ${expectedOffset}`);
assert.sameValue(instance.subtract(Temporal.Duration.from("P5432Y1835M")).offset, expectedOffset, `subtract -P5432Y1835M, offset should be ${expectedOffset}`);
