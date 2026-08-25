// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tostring
description: Verify that the year is appropriately formatted as 4 or 6 digits
features: [Temporal]
---*/

let instance = new Temporal.PlainDateTime(-100000, 12, 3, 4, 56, 7, 890);
assert.sameValue(instance.toString(), "-100000-12-03T04:56:07.89", "large negative year formatted as 6-digit");

instance = new Temporal.PlainDateTime(-10000, 4, 5, 6, 7, 8, 910);
assert.sameValue(instance.toString(), "-010000-04-05T06:07:08.91", "smallest 5-digit negative year formatted as 6-digit");

instance = new Temporal.PlainDateTime(-9999, 6, 7, 8, 9, 10, 987);
assert.sameValue(instance.toString(), "-009999-06-07T08:09:10.987", "largest 4-digit negative year formatted as 6-digit");

instance = new Temporal.PlainDateTime(-1000, 8, 9, 10, 9, 8, 765);
assert.sameValue(instance.toString(), "-001000-08-09T10:09:08.765", "smallest 4-digit negative year formatted as 6-digit");

instance = new Temporal.PlainDateTime(-999, 10, 9, 8, 7, 6, 543);
assert.sameValue(instance.toString(), "-000999-10-09T08:07:06.543", "largest 3-digit negative year formatted as 6-digit");

instance = new Temporal.PlainDateTime(-1, 8, 7, 6, 54, 32, 100);
assert.sameValue(instance.toString(), "-000001-08-07T06:54:32.1", "year -1 formatted as 6-digit");

instance = new Temporal.PlainDateTime(0, 6, 5, 4, 32, 10, 123);
assert.sameValue(instance.toString(), "0000-06-05T04:32:10.123", "year 0 formatted as 4-digit");

instance = new Temporal.PlainDateTime(1, 4, 3, 21, 0, 12, 345);
assert.sameValue(instance.toString(), "0001-04-03T21:00:12.345", "year 1 formatted as 4-digit");

instance = new Temporal.PlainDateTime(999, 2, 10, 12, 34, 56, 789);
assert.sameValue(instance.toString(), "0999-02-10T12:34:56.789", "largest 3-digit positive year formatted as 4-digit");

instance = new Temporal.PlainDateTime(1000, 1, 23, 4, 56, 7, 890);
assert.sameValue(instance.toString(), "1000-01-23T04:56:07.89", "smallest 4-digit positive year formatted as 4-digit");

instance = new Temporal.PlainDateTime(9999, 4, 5, 6, 7, 8, 910);
assert.sameValue(instance.toString(), "9999-04-05T06:07:08.91", "largest 4-digit positive year formatted as 4-digit");

instance = new Temporal.PlainDateTime(10000, 6, 7, 8, 9, 10, 987);
assert.sameValue(instance.toString(), "+010000-06-07T08:09:10.987", "smallest 5-digit positive year formatted as 6-digit");

instance = new Temporal.PlainDateTime(100000, 8, 9, 10, 9, 8, 765);
assert.sameValue(instance.toString(), "+100000-08-09T10:09:08.765", "large positive year formatted as 6-digit");
