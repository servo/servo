// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.equals
description: equals() ignores the observable properties and uses internal slots
features: [Temporal]
---*/

function CustomError() {}

class AvoidGettersYearMonth extends Temporal.PlainYearMonth {
  get year() {
    throw new CustomError();
  }
  get month() {
    throw new CustomError();
  }
}

const one = new AvoidGettersYearMonth(2000, 5);
const two = new AvoidGettersYearMonth(2006, 3);
assert.sameValue(one.equals(two), false);
