// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: String.prototype.indexOf type coercion for searchString parameter
esid: sec-string.prototype.indexof
info: |
  String.prototype.indexOf ( searchString [ , position ] )

  3. Let searchStr be ? ToString(searchString).
features: [Symbol.toPrimitive, computed-property-names]
---*/

assert.sameValue("__foo__".indexOf(Object("foo")), 2,
  "ToPrimitive: unbox object with internal slot");
assert.sameValue("__foo__".indexOf({
  [Symbol.toPrimitive]: function() {
    return "foo";
  }
}), 2, "ToPrimitive: @@toPrimitive");
assert.sameValue("__foo__".indexOf({
  valueOf: function() {
    return "foo";
  },
  toString: null
}), 2, "ToPrimitive: valueOf");
assert.sameValue("__foo__".indexOf({
  toString: function() {
    return "foo";
  }
}), 2, "ToPrimitive: toString");
assert.sameValue("__undefined__".indexOf({
  [Symbol.toPrimitive]: function() {
    return undefined;
  }
}), 2, 'ToString: @@toPrimitive => undefined => "undefined"');
assert.sameValue("__undefined__".indexOf({
  valueOf: function() {
    return undefined;
  },
  toString: null
}), 2, 'ToString: valueOf => undefined => "undefined"');
assert.sameValue("__undefined__".indexOf({
  toString: function() {
    return undefined;
  }
}), 2, 'ToString: toString => undefined => "undefined"');
assert.sameValue("__null__".indexOf({
  [Symbol.toPrimitive]: function() {
    return null;
  }
}), 2, 'ToString: @@toPrimitive => null => "null"');
assert.sameValue("__null__".indexOf({
  valueOf: function() {
    return null;
  },
  toString: null
}), 2, 'ToString: valueOf => null => "null"');
assert.sameValue("__null__".indexOf({
  toString: function() {
    return null;
  }
}), 2, 'ToString: toString => null => "null"');
assert.sameValue("__false__".indexOf(Object(false)), 2,
  'ToString: unbox object with internal slot => false => "false"');
assert.sameValue("__false__".indexOf({
  [Symbol.toPrimitive]: function() {
    return false;
  }
}), 2, 'ToString: @@toPrimitive => false => "false"');
assert.sameValue("__false__".indexOf({
  valueOf: function() {
    return false;
  },
  toString: null
}), 2, 'ToString: valueOf => false => "false"');
assert.sameValue("__false__".indexOf({
  toString: function() {
    return false;
  }
}), 2, 'ToString: toString => false => "false"');
assert.sameValue("__0__".indexOf(Object(0)), 2,
  "ToString: unbox object with internal slot => Number to String");
assert.sameValue("__0__".indexOf({
  [Symbol.toPrimitive]: function() {
    return 0;
  }
}), 2, "ToString: @@toPrimitive => Number to String");
assert.sameValue("__0__".indexOf({
  valueOf: function() {
    return 0;
  },
  toString: null
}), 2, "ToString: valueOf => Number to String");
assert.sameValue("__0__".indexOf({
  toString: function() {
    return 0;
  }
}), 2, "ToString: toString => Number to String");
