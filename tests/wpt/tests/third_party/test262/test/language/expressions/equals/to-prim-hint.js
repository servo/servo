// Copyright (C) 2017 Robin Templeton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-abstract-equality-comparison
description: Object operands coerced without ToPrimitive hint
info: |
  7.2.14 Abstract Equality Comparison

  ...
  6. If Type(x) is Boolean, return the result of the comparison !
  ToNumber(x) == y.
  7. If Type(y) is Boolean, return the result of the comparison x == !
  ToNumber(y).
  8. If Type(x) is either String, Number, or Symbol and Type(y) is
  Object, return the result of the comparison x == ToPrimitive(y).
  9. If Type(x) is Object and Type(y) is either String, Number, or
  Symbol, return the result of the comparison ToPrimitive(x) == y.
  ...
features: [Symbol.toPrimitive]
---*/

let count = 0;
let obj = {
  [Symbol.toPrimitive](hint) {
    count += 1;
    assert.sameValue(hint, "default");
    return 1;
  }
};

assert.sameValue(true == obj, true);
assert.sameValue(count, 1);
assert.sameValue(obj == true, true);
assert.sameValue(count, 2);
