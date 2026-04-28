// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.hoursinday
description: Handles dates with offset transitions where midnight occurs twice.
features: [Temporal]
---*/

// Test date with offset transition where the same day starts twice
// See https://github.com/tc39/proposal-temporal/issues/2938 for more details

const zdt1 = Temporal.ZonedDateTime.from('2010-11-06T00:00:00-02:30[America/St_Johns]');
const zdt2 = Temporal.ZonedDateTime.from('2010-11-07T23:00:00-03:30[America/St_Johns]');
const zdt3 = Temporal.ZonedDateTime.from('2010-11-08T23:00:00-03:30[America/St_Johns]');

assert.sameValue(zdt1.hoursInDay, 24);
assert.sameValue(zdt2.hoursInDay, 25);
assert.sameValue(zdt3.hoursInDay, 24);
