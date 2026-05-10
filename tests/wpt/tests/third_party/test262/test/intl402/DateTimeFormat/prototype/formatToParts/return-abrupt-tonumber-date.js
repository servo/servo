// Copyright 2016 Leonardo Balter. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: >
  Return abrupt completions from ToNumber(date)
info: |
  Intl.DateTimeFormat.prototype.formatToParts ([ date ])

  4. If _date_ is not provided or is *undefined*, then
    a. Let _x_ be *%Date_now%*().
  5. Else,
    a. Let _x_ be ? ToNumber(_date_).
features: [Symbol]
---*/

var obj1 = {
  valueOf: function() {
    throw new Test262Error();
  }
};

var obj2 = {
  toString: function() {
    throw new Test262Error();
  }
};

var dtf = new Intl.DateTimeFormat(["pt-BR"]);

assert.throws(Test262Error, function() {
  dtf.formatToParts(obj1);
}, "valueOf");

assert.throws(Test262Error, function() {
  dtf.formatToParts(obj2);
}, "toString");

var s = Symbol('1');
assert.throws(TypeError, function() {
  dtf.formatToParts(s);
}, "symbol");
