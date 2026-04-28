// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: String.prototype.indexOf type coercion for position parameter
esid: sec-string.prototype.indexof
info: |
  String.prototype.indexOf ( searchString [ , position ] )

  4. Let pos be ? ToInteger(position).
features: [Symbol.toPrimitive, computed-property-names]
---*/

assert.sameValue("aaaa".indexOf("aa", Object(0)), 0, "ToPrimitive: unbox object with internal slot");
assert.sameValue("aaaa".indexOf("aa", {
  [Symbol.toPrimitive]: function() {
    return 0;
  }
}), 0, "ToPrimitive: @@toPrimitive");
assert.sameValue("aaaa".indexOf("aa", {
  valueOf: function() {
    return 0;
  }
}), 0, "ToPrimitive: valueOf");
assert.sameValue("aaaa".indexOf("aa", {
  toString: function() {
    return 0;
  }
}), 0, "ToPrimitive: toString");
assert.sameValue("aaaa".indexOf("aa", Object(NaN)), 0,
  "ToInteger: unbox object with internal slot => NaN => 0");
assert.sameValue("aaaa".indexOf("aa", {
  [Symbol.toPrimitive]: function() {
    return NaN;
  }
}), 0, "ToInteger: @@toPrimitive => NaN => 0");
assert.sameValue("aaaa".indexOf("aa", {
  valueOf: function() {
    return NaN;
  }
}), 0, "ToInteger: valueOf => NaN => 0");
assert.sameValue("aaaa".indexOf("aa", {
  toString: function() {
    return NaN;
  }
}), 0, "ToInteger: toString => NaN => 0");
assert.sameValue("aaaa".indexOf("aa", {
  [Symbol.toPrimitive]: function() {
    return undefined;
  }
}), 0, "ToInteger: @@toPrimitive => undefined => NaN => 0");
assert.sameValue("aaaa".indexOf("aa", {
  valueOf: function() {
    return undefined;
  }
}), 0, "ToInteger: valueOf => undefined => NaN => 0");
assert.sameValue("aaaa".indexOf("aa", {
  toString: function() {
    return undefined;
  }
}), 0, "ToInteger: toString => undefined => NaN => 0");
assert.sameValue("aaaa".indexOf("aa", {
  [Symbol.toPrimitive]: function() {
    return null;
  }
}), 0, "ToInteger: @@toPrimitive => null => 0");
assert.sameValue("aaaa".indexOf("aa", {
  valueOf: function() {
    return null;
  }
}), 0, "ToInteger: valueOf => null => 0");
assert.sameValue("aaaa".indexOf("aa", {
  toString: function() {
    return null;
  }
}), 0, "ToInteger: toString => null => 0");
assert.sameValue("aaaa".indexOf("aa", Object(true)), 1,
  "ToInteger: unbox object with internal slot => true => 1");
assert.sameValue("aaaa".indexOf("aa", {
  [Symbol.toPrimitive]: function() {
    return true;
  }
}), 1, "ToInteger: @@toPrimitive => true => 1");
assert.sameValue("aaaa".indexOf("aa", {
  valueOf: function() {
    return true;
  }
}), 1, "ToInteger: valueOf => true => 1");
assert.sameValue("aaaa".indexOf("aa", {
  toString: function() {
    return true;
  }
}), 1, "ToInteger: toString => true => 1");
assert.sameValue("aaaa".indexOf("aa", Object("1.9")), 1,
  "ToInteger: unbox object with internal slot => parse Number => 1.9 => 1");
assert.sameValue("aaaa".indexOf("aa", {
  [Symbol.toPrimitive]: function() {
    return "1.9";
  }
}), 1, "ToInteger: @@toPrimitive => parse Number => 1.9 => 1");
assert.sameValue("aaaa".indexOf("aa", {
  valueOf: function() {
    return "1.9";
  }
}), 1, "ToInteger: valueOf => parse Number => 1.9 => 1");
assert.sameValue("aaaa".indexOf("aa", {
  toString: function() {
    return "1.9";
  }
}), 1, "ToInteger: toString => parse Number => 1.9 => 1");
