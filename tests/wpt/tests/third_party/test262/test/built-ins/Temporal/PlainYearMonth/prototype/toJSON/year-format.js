// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.tojson
description: Verify that the year is appropriately formatted as 4 or 6 digits
features: [Temporal]
---*/

let instance = new Temporal.PlainYearMonth(-100000, 12);
assert.sameValue(instance.toJSON(), "-100000-12", "large negative year formatted as 6-digit");

instance = new Temporal.PlainYearMonth(-10000, 4);
assert.sameValue(instance.toJSON(), "-010000-04", "smallest 5-digit negative year formatted as 6-digit");

instance = new Temporal.PlainYearMonth(-9999, 6);
assert.sameValue(instance.toJSON(), "-009999-06", "largest 4-digit negative year formatted as 6-digit");

instance = new Temporal.PlainYearMonth(-1000, 8);
assert.sameValue(instance.toJSON(), "-001000-08", "smallest 4-digit negative year formatted as 6-digit");

instance = new Temporal.PlainYearMonth(-999, 10);
assert.sameValue(instance.toJSON(), "-000999-10", "largest 3-digit negative year formatted as 6-digit");

instance = new Temporal.PlainYearMonth(-1, 8);
assert.sameValue(instance.toJSON(), "-000001-08", "year -1 formatted as 6-digit");

instance = new Temporal.PlainYearMonth(0, 6);
assert.sameValue(instance.toJSON(), "0000-06", "year 0 formatted as 4-digit");

instance = new Temporal.PlainYearMonth(1, 4);
assert.sameValue(instance.toJSON(), "0001-04", "year 1 formatted as 4-digit");

instance = new Temporal.PlainYearMonth(999, 2);
assert.sameValue(instance.toJSON(), "0999-02", "largest 3-digit positive year formatted as 4-digit");

instance = new Temporal.PlainYearMonth(1000, 1);
assert.sameValue(instance.toJSON(), "1000-01", "smallest 4-digit positive year formatted as 4-digit");

instance = new Temporal.PlainYearMonth(9999, 4);
assert.sameValue(instance.toJSON(), "9999-04", "largest 4-digit positive year formatted as 4-digit");

instance = new Temporal.PlainYearMonth(10000, 6);
assert.sameValue(instance.toJSON(), "+010000-06", "smallest 5-digit positive year formatted as 6-digit");

instance = new Temporal.PlainYearMonth(100000, 8);
assert.sameValue(instance.toJSON(), "+100000-08", "large positive year formatted as 6-digit");
