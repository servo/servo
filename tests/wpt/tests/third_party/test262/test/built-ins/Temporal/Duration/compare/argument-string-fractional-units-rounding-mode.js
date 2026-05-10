// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.compare
description: Strings with fractional duration units are rounded with the correct rounding mode
features: [Temporal]
---*/

const expectedPos = new Temporal.Duration(0, 0, 0, 0, 1, 1, 52, 500);
const expectedNeg = new Temporal.Duration(0, 0, 0, 0, -1, -1, -52, -500);

assert.sameValue(Temporal.Duration.compare("PT1.03125H", expectedPos), 0,
  "positive fractional units rounded with correct rounding mode (first argument)");
assert.sameValue(Temporal.Duration.compare("-PT1.03125H", expectedNeg), 0,
  "negative fractional units rounded with correct rounding mode (first argument)");
assert.sameValue(Temporal.Duration.compare(expectedPos, "PT1.03125H"), 0,
  "positive fractional units rounded with correct rounding mode (second argument)");
assert.sameValue(Temporal.Duration.compare(expectedNeg, "-PT1.03125H"), 0,
  "negative fractional units rounded with correct rounding mode (second argument)");
