// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: String.prototype.indexOf type coercion for searchString parameter
esid: sec-string.prototype.indexof
info: |
  String.prototype.indexOf ( searchString [ , position ] )

  3. Let searchStr be ? ToString(searchString).
features: [Symbol, Symbol.toPrimitive, computed-property-names]
---*/

assert.throws(TypeError, function() {
  "".indexOf(Symbol("1"));
}, "ToString: Symbol => TypeError");
assert.throws(TypeError, function() {
  "".indexOf(Object(Symbol("1")));
}, "ToString: unbox object with internal slot => Symbol => TypeError");
assert.throws(TypeError, function() {
  "".indexOf({
    [Symbol.toPrimitive]: function() {
      return Symbol("1");
    }
  });
}, "ToString: @@toPrimitive => Symbol => TypeError");
assert.throws(TypeError, function() {
  "".indexOf({
    valueOf: function() {
      return Symbol("1");
    },
    toString: null
  });
}, "ToString: valueOf => Symbol => TypeError");
assert.throws(TypeError, function() {
  "".indexOf({
    toString: function() {
      return Symbol("1");
    }
  });
}, "ToString: toString => Symbol => TypeError");
