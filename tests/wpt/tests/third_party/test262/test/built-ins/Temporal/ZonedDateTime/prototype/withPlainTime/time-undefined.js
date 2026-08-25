// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.withplaintime
description: The time is assumed to be midnight if not given
features: [BigInt, Temporal]
---*/

const datetime = new Temporal.ZonedDateTime(957270896_987_654_321n, "UTC");

const explicit = datetime.withPlainTime(undefined);
assert.sameValue(explicit.hour, 0, "default time is midnight");
assert.sameValue(explicit.minute, 0, "default time is midnight");
assert.sameValue(explicit.second, 0, "default time is midnight");
assert.sameValue(explicit.millisecond, 0, "default time is midnight");
assert.sameValue(explicit.microsecond, 0, "default time is midnight");
assert.sameValue(explicit.nanosecond, 0, "default time is midnight");

const implicit = datetime.withPlainTime();
assert.sameValue(implicit.hour, 0, "default time is midnight");
assert.sameValue(implicit.minute, 0, "default time is midnight");
assert.sameValue(implicit.second, 0, "default time is midnight");
assert.sameValue(implicit.millisecond, 0, "default time is midnight");
assert.sameValue(implicit.microsecond, 0, "default time is midnight");
assert.sameValue(implicit.nanosecond, 0, "default time is midnight");
