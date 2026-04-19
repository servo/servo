// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.tostring
description: Fallback value for roundingMode option
features: [Temporal]
---*/

const duration = new Temporal.Duration(0, 0, 0, 0, 12, 34, 56, 123, 987, 500);

const explicit1 = duration.toString({ smallestUnit: "microsecond", roundingMode: undefined });
assert.sameValue(explicit1, "PT12H34M56.123987S", "default roundingMode is trunc");
const implicit1 = duration.toString({ smallestUnit: "microsecond" });
assert.sameValue(implicit1, "PT12H34M56.123987S", "default roundingMode is trunc");

const explicit2 = duration.toString({ smallestUnit: "millisecond", roundingMode: undefined });
assert.sameValue(explicit2, "PT12H34M56.123S", "default roundingMode is trunc");
const implicit2 = duration.toString({ smallestUnit: "millisecond" });
assert.sameValue(implicit2, "PT12H34M56.123S", "default roundingMode is trunc");

const explicit3 = duration.toString({ smallestUnit: "second", roundingMode: undefined });
assert.sameValue(explicit3, "PT12H34M56S", "default roundingMode is trunc");
const implicit3 = duration.toString({ smallestUnit: "second" });
assert.sameValue(implicit3, "PT12H34M56S", "default roundingMode is trunc");
