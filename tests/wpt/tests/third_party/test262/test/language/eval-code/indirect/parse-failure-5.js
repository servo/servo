// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
info: |
    If Result(3).type is not normal, then Result(3).type must be throw.
    Throw Result(3).value as an exception
esid: sec-performeval
es5id: 15.1.2.1_A3.3_T3
description: Return statement
---*/

var value;

try {
  value = (0,eval)("return;");
  throw new Test262Error('#1.1: return must throw SyntaxError. Actual: ' + value);
} catch(e) {
  if ((e instanceof SyntaxError) !== true) {
    throw new Test262Error('#1.2: return must throw SyntaxError. Actual: ' + e);
  }
}

assert.throws(SyntaxError, function() {
  (0,eval)("return;");
});
