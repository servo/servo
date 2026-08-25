// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.tostring
description: Fallback value for smallestUnit option
features: [Temporal]
---*/

const duration = new Temporal.Duration(0, 0, 0, 0, 12, 34, 56, 123, 987, 500);

const explicit1 = duration.toString({ smallestUnit: undefined, fractionalSecondDigits: 6 });
assert.sameValue(explicit1, "PT12H34M56.123987S", "default smallestUnit defers to fractionalSecondDigits");
const implicit1 = duration.toString({ fractionalSecondDigits: 6 });
assert.sameValue(implicit1, "PT12H34M56.123987S", "default smallestUnit defers to fractionalSecondDigits");

const explicit2 = duration.toString({ smallestUnit: undefined, fractionalSecondDigits: 3 });
assert.sameValue(explicit2, "PT12H34M56.123S", "default smallestUnit defers to fractionalSecondDigits");
const implicit2 = duration.toString({ fractionalSecondDigits: 3 });
assert.sameValue(implicit2, "PT12H34M56.123S", "default smallestUnit defers to fractionalSecondDigits");
