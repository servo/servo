// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: String.prototype.indexOf type coercion for position parameter
esid: sec-string.prototype.indexof
info: |
  String.prototype.indexOf ( searchString [ , position ] )

  4. Let pos be ? ToInteger(position).
features: [Symbol, Symbol.toPrimitive, computed-property-names]
---*/

assert.throws(TypeError, function() {
  "".indexOf("", Symbol("1"));
}, "ToInteger: Symbol => TypeError");
assert.throws(TypeError, function() {
  "".indexOf("", Object(Symbol("1")));
}, "ToInteger: unbox object with internal slot => Symbol => TypeError");
assert.throws(TypeError, function() {
  "".indexOf("", {
    [Symbol.toPrimitive]: function() {
      return Symbol("1");
    }
  });
}, "ToInteger: @@toPrimitive => Symbol => TypeError");
assert.throws(TypeError, function() {
  "".indexOf("", {
    valueOf: function() {
      return Symbol("1");
    }
  });
}, "ToInteger: valueOf => Symbol => TypeError");
assert.throws(TypeError, function() {
  "".indexOf("", {
    toString: function() {
      return Symbol("1");
    }
  });
}, "ToInteger: toString => Symbol => TypeError");
