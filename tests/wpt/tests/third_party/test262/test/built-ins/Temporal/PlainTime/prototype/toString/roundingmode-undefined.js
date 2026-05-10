// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.tostring
description: Fallback value for roundingMode option
features: [Temporal]
---*/

const time = new Temporal.PlainTime(12, 34, 56, 123, 987, 500);

const explicit1 = time.toString({ smallestUnit: "microsecond", roundingMode: undefined });
assert.sameValue(explicit1, "12:34:56.123987", "default roundingMode is trunc");
const implicit1 = time.toString({ smallestUnit: "microsecond" });
assert.sameValue(implicit1, "12:34:56.123987", "default roundingMode is trunc");

const explicit2 = time.toString({ smallestUnit: "millisecond", roundingMode: undefined });
assert.sameValue(explicit2, "12:34:56.123", "default roundingMode is trunc");
const implicit2 = time.toString({ smallestUnit: "millisecond" });
assert.sameValue(implicit2, "12:34:56.123", "default roundingMode is trunc");

const explicit3 = time.toString({ smallestUnit: "second", roundingMode: undefined });
assert.sameValue(explicit3, "12:34:56", "default roundingMode is trunc");
const implicit3 = time.toString({ smallestUnit: "second" });
assert.sameValue(implicit3, "12:34:56", "default roundingMode is trunc");
