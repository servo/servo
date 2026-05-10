// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.tostring
description: Verify that the year is appropriately formatted as 4 or 6 digits
features: [Temporal]
---*/

let instance = new Temporal.PlainDate(-100000, 12, 3);
assert.sameValue(instance.toString(), "-100000-12-03", "large negative year formatted as 6-digit");

instance = new Temporal.PlainDate(-10000, 4, 5);
assert.sameValue(instance.toString(), "-010000-04-05", "smallest 5-digit negative year formatted as 6-digit");

instance = new Temporal.PlainDate(-9999, 6, 7);
assert.sameValue(instance.toString(), "-009999-06-07", "largest 4-digit negative year formatted as 6-digit");

instance = new Temporal.PlainDate(-1000, 8, 9);
assert.sameValue(instance.toString(), "-001000-08-09", "smallest 4-digit negative year formatted as 6-digit");

instance = new Temporal.PlainDate(-999, 10, 9);
assert.sameValue(instance.toString(), "-000999-10-09", "largest 3-digit negative year formatted as 6-digit");

instance = new Temporal.PlainDate(-1, 8, 7);
assert.sameValue(instance.toString(), "-000001-08-07", "year -1 formatted as 6-digit");

instance = new Temporal.PlainDate(0, 6, 5);
assert.sameValue(instance.toString(), "0000-06-05", "year 0 formatted as 4-digit");

instance = new Temporal.PlainDate(1, 4, 3);
assert.sameValue(instance.toString(), "0001-04-03", "year 1 formatted as 4-digit");

instance = new Temporal.PlainDate(999, 2, 10);
assert.sameValue(instance.toString(), "0999-02-10", "largest 3-digit positive year formatted as 4-digit");

instance = new Temporal.PlainDate(1000, 1, 23);
assert.sameValue(instance.toString(), "1000-01-23", "smallest 4-digit positive year formatted as 4-digit");

instance = new Temporal.PlainDate(9999, 4, 5);
assert.sameValue(instance.toString(), "9999-04-05", "largest 4-digit positive year formatted as 4-digit");

instance = new Temporal.PlainDate(10000, 6, 7);
assert.sameValue(instance.toString(), "+010000-06-07", "smallest 5-digit positive year formatted as 6-digit");

instance = new Temporal.PlainDate(100000, 8, 9);
assert.sameValue(instance.toString(), "+100000-08-09", "large positive year formatted as 6-digit");
