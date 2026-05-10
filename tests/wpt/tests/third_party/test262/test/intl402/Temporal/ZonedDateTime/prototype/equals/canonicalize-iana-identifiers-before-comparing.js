// Copyright (C) 2023 Justin Grant. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.equals
description: ZonedDateTime.p.equals canonicalizes time zone IDs before comparing them
features: [Temporal]
---*/

const calcutta = Temporal.ZonedDateTime.from('2020-01-01T00:00:00+05:30[Asia/Calcutta]');
const kolkata = Temporal.ZonedDateTime.from('2020-01-01T00:00:00+05:30[Asia/Kolkata]');
const colombo = Temporal.ZonedDateTime.from('2020-01-01T00:00:00+05:30[Asia/Colombo]');

assert.sameValue(calcutta.equals(kolkata), true);
assert.sameValue(calcutta.equals(kolkata.toString()), true);
assert.sameValue(kolkata.equals(calcutta), true);
assert.sameValue(kolkata.equals(calcutta.toString()), true);
assert.sameValue(calcutta.equals(colombo), false);
