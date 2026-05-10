// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.tolocalestring
description: Omitting the locales argument defaults to the DateTimeFormat default
features: [Temporal]
---*/

const time = new Temporal.PlainTime(12, 34, 56, 987, 654, 321);
const defaultFormatter = new Intl.DateTimeFormat([], Object.create(null));
const expected = defaultFormatter.format(time);

const actualExplicit = time.toLocaleString(undefined);
assert.sameValue(actualExplicit, expected, "default locale is determined by Intl.DateTimeFormat");

const actualImplicit = time.toLocaleString();
assert.sameValue(actualImplicit, expected, "default locale is determined by Intl.DateTimeFormat");
