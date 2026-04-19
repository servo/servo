// Copyright 2019 Google, Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Throws a TypeError if startDate or endDate is undefined.
info: |
  Intl.DateTimeFormat.prototype.formatRange ( startDate , endDate )

  1. Let dtf be this value.
  2. If Type(dtf) is not Object, throw a TypeError exception.
  3. If dtf does not have an [[InitializedDateTimeFormat]] internal slot, throw a TypeError exception.
  4. If startDate is undefined or endDate is undefined, throw a TypeError exception.
  5. Let x be ? ToNumber(startDate).
  6. Let y be ? ToNumber(endDate).

features: [Intl.DateTimeFormat-formatRange]
---*/

var dtf = new Intl.DateTimeFormat();

assert.throws(TypeError, function() {
  dtf.formatRange(); // Not possible to poison this one
}, "no args");

var poison = { valueOf() { throw new Test262Error(); } };

assert.throws(TypeError, function() {
  dtf.formatRange(undefined, poison);
}, "date/undefined");

assert.throws(TypeError, function() {
  dtf.formatRange(poison, undefined);
}, "undefined/date");

assert.throws(TypeError, function() {
  dtf.formatRange(poison);
}, "only one arg");

assert.throws(TypeError, function() {
  dtf.formatRange(undefined, undefined);
}, "undefined/undefined");
