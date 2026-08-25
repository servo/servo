// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tolocalestring
description: Omitting the locales argument defaults to the DateTimeFormat default
features: [BigInt, Temporal]
---*/

const instant = new Temporal.Instant(957270896_987_650_000n);
const defaultFormatter = new Intl.DateTimeFormat([], Object.create(null));
const expected = defaultFormatter.format(instant);

const actualExplicit = instant.toLocaleString(undefined);
assert.sameValue(actualExplicit, expected, "default locale is determined by Intl.DateTimeFormat");

const actualImplicit = instant.toLocaleString();
assert.sameValue(actualImplicit, expected, "default locale is determined by Intl.DateTimeFormat");
