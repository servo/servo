// Copyright 2019 Igalia S.L. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: >
  Return abrupt completions from ToNumber(date)
info: |
  Intl.DateTimeFormat.prototype.formatRangeToParts ( startDate , endDate )

  5. Let x be ? ToNumber(startDate).
  6. Let y be ? ToNumber(endDate).
features: [Symbol,Intl.DateTimeFormat-formatRange]
---*/

const date = Date.now();

const objectValueOf = {
  valueOf: function() {
    throw new Test262Error();
  }
};

const objectToString = {
  toString: function() {
    throw new Test262Error();
  }
};

const dtf = new Intl.DateTimeFormat(["pt-BR"]);

assert.throws(Test262Error, function() {
  dtf.formatRangeToParts(objectValueOf, date);
}, "valueOf start");

assert.throws(Test262Error, function() {
  dtf.formatRangeToParts(date, objectValueOf);
}, "valueOf end");

assert.throws(Test262Error, function() {
  dtf.formatRangeToParts(objectToString, date);
}, "toString start");

assert.throws(Test262Error, function() {
  dtf.formatRangeToParts(date, objectToString);
}, "toString end");

const s = Symbol('1');
assert.throws(TypeError, function() {
  dtf.formatRangeToParts(s, date);
}, "symbol start");

assert.throws(TypeError, function() {
  dtf.formatRangeToParts(date, s);
}, "symbol end");
