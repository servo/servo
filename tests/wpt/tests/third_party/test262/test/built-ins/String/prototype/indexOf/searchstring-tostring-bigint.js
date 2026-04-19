// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: String.prototype.indexOf type coercion for searchString parameter
esid: sec-string.prototype.indexof
info: |
  String.prototype.indexOf ( searchString [ , position ] )

  3. Let searchStr be ? ToString(searchString).
features: [BigInt, Symbol.toPrimitive, computed-property-names]
---*/

assert.sameValue("__0__".indexOf(0n), 2, "ToString: BigInt to String");
assert.sameValue("__0__".indexOf(Object(0n)), 2,
  "ToString: unbox object with internal slot => BigInt to String");
assert.sameValue("__0__".indexOf({
  [Symbol.toPrimitive]: function() {
    return 0n;
  }
}), 2, "ToString: @@toPrimitive => BigInt to String");
assert.sameValue("__0__".indexOf({
  valueOf: function() {
    return 0n;
  },
  toString: null
}), 2, "ToString: valueOf => BigInt to String");
assert.sameValue("__0__".indexOf({
  toString: function() {
    return 0n;
  }
}), 2, "ToString: toString => BigInt to String");
