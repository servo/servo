// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tostring
description: Fallback value for roundingMode option
features: [Temporal]
---*/

const datetime = new Temporal.PlainDateTime(2000, 5, 2, 12, 34, 56, 123, 987, 500);

const explicit1 = datetime.toString({ smallestUnit: "microsecond", roundingMode: undefined });
assert.sameValue(explicit1, "2000-05-02T12:34:56.123987", "default roundingMode is trunc");
const implicit1 = datetime.toString({ smallestUnit: "microsecond" });
assert.sameValue(implicit1, "2000-05-02T12:34:56.123987", "default roundingMode is trunc");

const explicit2 = datetime.toString({ smallestUnit: "millisecond", roundingMode: undefined });
assert.sameValue(explicit2, "2000-05-02T12:34:56.123", "default roundingMode is trunc");
const implicit2 = datetime.toString({ smallestUnit: "millisecond" });
assert.sameValue(implicit2, "2000-05-02T12:34:56.123", "default roundingMode is trunc");

const explicit3 = datetime.toString({ smallestUnit: "second", roundingMode: undefined });
assert.sameValue(explicit3, "2000-05-02T12:34:56", "default roundingMode is trunc");
const implicit3 = datetime.toString({ smallestUnit: "second" });
assert.sameValue(implicit3, "2000-05-02T12:34:56", "default roundingMode is trunc");
