// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.tolocalestring
description: Temporal.PlainTime should be interpreted and formatted as wall-clock time
features: [Temporal]
---*/

// A non-existent time in the 'America/Los_Angeles' timezone.
const instance1 = Temporal.PlainTime.from('2026-03-29T02:30:00');

assert.sameValue(
  instance1.toLocaleString('en-US'),
  instance1.toLocaleString('en-US', { timeZone: 'America/Los_Angeles' })
)

const result1 = instance1.toLocaleString('en-US', { timeZone: 'America/Los_Angeles' });
assert(result1.includes('2:30') && !result1.includes('3:'))

assert.sameValue(
  instance1.toLocaleString('en-US', { timeZone: 'UTC' }),
  instance1.toLocaleString('en-US', { timeZone: 'America/Los_Angeles' })
)

// Creating the instance from a datestring with an offset has no effect.

const instance2 = Temporal.PlainTime.from('2026-03-29T02:30:15+01:00');

const result2 = instance2.toLocaleString('en', { timeStyle: 'long' });
assert(result2.includes('2:30') && !result2.includes('3:'));

assert.sameValue(
  instance2.toLocaleString('en', { timeStyle: 'long' }),
  instance2.toLocaleString('en', { timeStyle: 'long', timeZone: 'America/New_York' }),
);
