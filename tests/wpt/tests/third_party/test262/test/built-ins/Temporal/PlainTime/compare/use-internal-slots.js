// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal-comparetemporaltime
description: compare() ignores the observable properties and uses internal slots
features: [Temporal]
---*/

function CustomError() {}

class AvoidGettersTime extends Temporal.PlainTime {
  get hour() {
    throw new CustomError();
  }
  get minute() {
    throw new CustomError();
  }
  get second() {
    throw new CustomError();
  }
  get millisecond() {
    throw new CustomError();
  }
  get microsecond() {
    throw new CustomError();
  }
  get nanosecond() {
    throw new CustomError();
  }
}

const one = new AvoidGettersTime(12, 34, 56, 987, 654, 321);
const two = new AvoidGettersTime(6, 54, 32, 123, 456, 789);
assert.sameValue(Temporal.PlainTime.compare(one, two), 1);
