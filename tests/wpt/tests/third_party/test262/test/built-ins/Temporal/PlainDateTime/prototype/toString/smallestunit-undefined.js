// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tostring
description: Fallback value for smallestUnit option
features: [Temporal]
---*/

const datetime = new Temporal.PlainDateTime(2000, 5, 2, 12, 34, 56, 123, 987, 500);

const explicit1 = datetime.toString({ smallestUnit: undefined, fractionalSecondDigits: 6 });
assert.sameValue(explicit1, "2000-05-02T12:34:56.123987", "default smallestUnit defers to fractionalSecondDigits");
const implicit1 = datetime.toString({ fractionalSecondDigits: 6 });
assert.sameValue(implicit1, "2000-05-02T12:34:56.123987", "default smallestUnit defers to fractionalSecondDigits");

const explicit2 = datetime.toString({ smallestUnit: undefined, fractionalSecondDigits: 3 });
assert.sameValue(explicit2, "2000-05-02T12:34:56.123", "default smallestUnit defers to fractionalSecondDigits");
const implicit2 = datetime.toString({ fractionalSecondDigits: 3 });
assert.sameValue(implicit2, "2000-05-02T12:34:56.123", "default smallestUnit defers to fractionalSecondDigits");
