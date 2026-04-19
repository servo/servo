// Copyright (C) 2023 Justin Grant. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.until
description: Accept time zone identifiers that canonicalize to the same ID
features: [Temporal]
---*/

const calcutta = Temporal.ZonedDateTime.from('2020-01-01T00:00:00+05:30[Asia/Calcutta]');
const kolkata = Temporal.ZonedDateTime.from('2021-09-01T00:00:00+05:30[Asia/Kolkata]');
const colombo = Temporal.ZonedDateTime.from('2022-08-01T00:00:00+05:30[Asia/Colombo]');

// If the time zones resolve to the same canonical zone, then it shouldn't throw
assert.sameValue(calcutta.until(kolkata, { largestUnit: 'day' }).toString(), 'P609D');
assert.throws(RangeError, () => calcutta.until(colombo, { largestUnit: 'day' }));
