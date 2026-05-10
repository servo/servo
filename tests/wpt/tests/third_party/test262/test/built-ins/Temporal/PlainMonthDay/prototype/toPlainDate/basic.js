// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.toplaindate
description: Basic tests for toPlainDate().
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const md = Temporal.PlainMonthDay.from("01-22");
const d = md.toPlainDate({ year: 2002 });
TemporalHelpers.assertPlainDate(d, 2002, 1, "M01", 22);

assert.throws(TypeError, () => md.toPlainDate({ something: 'nothing' }), "missing fields");

const leapDay = Temporal.PlainMonthDay.from('02-29');
TemporalHelpers.assertPlainDate(leapDay.toPlainDate({ year: 2020 }), 2020, 2, "M02", 29);

const options = {
  get overflow() {
    TemporalHelpers.assertUnreachable("Should not get overflow option");
    return "";
  }
};
TemporalHelpers.assertPlainDate(leapDay.toPlainDate({ year: 2020 }, options), 2020, 2, "M02", 29);
