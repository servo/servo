// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.compare
description: Leap second is a valid ISO string for ZonedDateTime
features: [Temporal]
---*/

const datetime = new Temporal.ZonedDateTime(1_483_228_799_000_000_000n, "UTC");

let arg = "2016-12-31T23:59:60+00:00[UTC]";
const result1 = Temporal.ZonedDateTime.compare(arg, datetime);
assert.sameValue(result1, 0, "leap second is a valid ISO string for ZonedDateTime (first argument)");
const result2 = Temporal.ZonedDateTime.compare(datetime, arg);
assert.sameValue(result2, 0, "leap second is a valid ISO string for ZonedDateTime (second argument)");

arg = "2000-05-02T12:34:56+23:59[+23:59:60]";
assert.throws(RangeError, () => Temporal.ZonedDateTime.compare(arg, datetime), "leap second in time zone name not valid (first argument)");
assert.throws(RangeError, () => Temporal.ZonedDateTime.compare(datetime, arg), "leap second in time zone name not valid (second argument)");
