// Copyright (C) 2020 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal-compareisodatetime
description: compare() ignores the observable properties and uses internal slots
features: [Temporal]
---*/

function CustomError() {}

class AvoidGettersDateTime extends Temporal.PlainDateTime {
  get year() {
    throw new CustomError();
  }
  get month() {
    throw new CustomError();
  }
  get day() {
    throw new CustomError();
  }
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

const one = new AvoidGettersDateTime(2000, 5, 2, 12, 34, 56, 987, 654, 321);
const two = new AvoidGettersDateTime(2006, 3, 25, 6, 54, 32, 123, 456, 789);
assert.sameValue(Temporal.PlainDateTime.compare(one, two), -1);
