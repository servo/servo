// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tolocalestring
description: Verify that undefined options are handled correctly.
features: [Temporal]
---*/

const datetime = new Temporal.PlainDateTime(2000, 5, 2, 12, 34, 56, 987, 654, 321);
const defaultFormatter = new Intl.DateTimeFormat('en', Object.create(null));
const expected = defaultFormatter.format(datetime);

const actualExplicit = datetime.toLocaleString('en', undefined);
assert.sameValue(actualExplicit, expected, "default locale is determined by Intl.DateTimeFormat");

const actualImplicit = datetime.toLocaleString('en');
assert.sameValue(actualImplicit, expected, "default locale is determined by Intl.DateTimeFormat");
