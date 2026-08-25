// Copyright (C) 2023 Justin Grant. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: ZonedDateTime.from does not canonicalize time zone IDs
features: [Temporal, canonical-tz]
---*/

const calcutta = Temporal.ZonedDateTime.from('2020-01-01T00:00:00+05:30[Asia/Calcutta]');
const kolkata = Temporal.ZonedDateTime.from('2020-01-01T00:00:00+05:30[Asia/Kolkata]');

assert.sameValue(calcutta.toString(), '2020-01-01T00:00:00+05:30[Asia/Calcutta]');
assert.sameValue(calcutta.toJSON(), '2020-01-01T00:00:00+05:30[Asia/Calcutta]');
assert.sameValue(calcutta.timeZoneId, 'Asia/Calcutta');

assert.sameValue(kolkata.toString(), '2020-01-01T00:00:00+05:30[Asia/Kolkata]');
assert.sameValue(kolkata.toJSON(), '2020-01-01T00:00:00+05:30[Asia/Kolkata]');
assert.sameValue(kolkata.timeZoneId, 'Asia/Kolkata');
