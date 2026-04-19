// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.tostring
description: Fallback value for smallestUnit option
features: [Temporal]
---*/

const time = new Temporal.PlainTime(12, 34, 56, 123, 987, 500);

const explicit1 = time.toString({ smallestUnit: undefined, fractionalSecondDigits: 6 });
assert.sameValue(explicit1, "12:34:56.123987", "default smallestUnit defers to fractionalSecondDigits");
const implicit1 = time.toString({ fractionalSecondDigits: 6 });
assert.sameValue(implicit1, "12:34:56.123987", "default smallestUnit defers to fractionalSecondDigits");

const explicit2 = time.toString({ smallestUnit: undefined, fractionalSecondDigits: 3 });
assert.sameValue(explicit2, "12:34:56.123", "default smallestUnit defers to fractionalSecondDigits");
const implicit2 = time.toString({ fractionalSecondDigits: 3 });
assert.sameValue(implicit2, "12:34:56.123", "default smallestUnit defers to fractionalSecondDigits");
