// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.compare
description: >
    The relativeTo option is required when either Duration contains years,
    months, or weeks
features: [Temporal]
---*/

const oneYear = new Temporal.Duration(1);
const oneMonth = new Temporal.Duration(0, 1);
const oneWeek = new Temporal.Duration(0, 0, 1);
const oneDay = new Temporal.Duration(0, 0, 0, 1);
const twoDays = new Temporal.Duration(0, 0, 0, 2);

assert.sameValue(Temporal.Duration.compare(oneDay, twoDays), -1, "days do not require relativeTo");

assert.throws(RangeError, () => Temporal.Duration.compare(oneWeek, oneDay), "weeks in left operand require relativeTo");
assert.throws(RangeError, () => Temporal.Duration.compare(oneDay, oneWeek), "weeks in right operand require relativeTo");

assert.throws(RangeError, () => Temporal.Duration.compare(oneMonth, oneDay), "months in left operand require relativeTo");
assert.throws(RangeError, () => Temporal.Duration.compare(oneDay, oneMonth), "months in right operand require relativeTo");

assert.throws(RangeError, () => Temporal.Duration.compare(oneYear, oneDay), "years in left operand require relativeTo");
assert.throws(RangeError, () => Temporal.Duration.compare(oneDay, oneYear), "years in right operand require relativeTo");
