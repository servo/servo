// Copyright 2019 Google, Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Throws a TypeError if this is not Object.
info: |
  Intl.DateTimeFormat.prototype.formatRangeToParts ( startDate , endDate )

  1. Let dtf be this value.
  2. If Type(dtf) is not Object, throw a TypeError exception.

features: [Intl.DateTimeFormat-formatRange, Symbol]
---*/

let formatRangeToParts = Intl.DateTimeFormat.prototype.formatRangeToParts;
let d1 = new Date("1997-08-22T00:00");
let d2 = new Date("1999-06-26T00:00");

assert.throws(TypeError, function() {
  formatRangeToParts.call(undefined, d1, d2);
}, "undefined");

assert.throws(TypeError, function() {
  formatRangeToParts.call(null, d1, d2);
}, "null");

assert.throws(TypeError, function() {
  formatRangeToParts.call(42, d1, d2);
}, "number");

assert.throws(TypeError, function() {
  formatRangeToParts.call("foo", d1, d2);
}, "string");

assert.throws(TypeError, function() {
  formatRangeToParts.call(false, d1, d2);
}, "false");

assert.throws(TypeError, function() {
  formatRangeToParts.call(true, d1, d2);
}, "true");

var s = Symbol('3');
assert.throws(TypeError, function() {
  formatRangeToParts.call(s, d1, d2);
}, "symbol");
