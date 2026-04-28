// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-symbol-constructor
description: The first argument is coerced to a String value (from a Symbol)
info: |
    1. If NewTarget is not undefined, throw a TypeError exception.
    2. If description is undefined, let descString be undefined.
    2. Else, let descString be ? ToString(description).
    3. Return a new unique Symbol value whose [[Description]] value is
       descString.
features: [Symbol]
---*/

var s = Symbol('1');

assert.throws(TypeError, function() {
  Symbol(s);
});
