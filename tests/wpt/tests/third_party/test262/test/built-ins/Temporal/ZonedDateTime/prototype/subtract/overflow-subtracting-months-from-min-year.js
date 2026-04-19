// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.subtract
description: Subtracting months from minimum year should throw
features: [Temporal]
---*/

const minYear = new Temporal.ZonedDateTime(-(864n * 10n ** 19n), "UTC");
const duration = new Temporal.Duration(0, 5432, 5432, 0, 0, 0, 0, 0, 0, 0);
assert.throws(RangeError, () => minYear.subtract(duration));

const maxYear = new Temporal.PlainDateTime(275760, 1, 1).toZonedDateTime("UTC");
assert.throws(RangeError, () => maxYear.subtract(duration.negated()));
