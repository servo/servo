// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-class-definitions-static-semantics-early-errors
description: Returns abrupt completion and halts further class body evaluation
info: |
  34. For each element elementRecord of staticElements in List order, do
      a. If elementRecord is a ClassFieldDefinition Record, then
         [...]
      b. Else,
         i. Assert: fieldRecord is a ClassStaticBlockDefinition Record.
         ii. Let status be the result of performing EvaluateStaticBlock(F,
             elementRecord).
      d. If status is an abrupt completion, then
          i. Set the running execution context's LexicalEnvironment to lex.
          ii. Set the running execution context's PrivateEnvironment to
              outerPrivateEnvironment.
          iii. Return Completion(status).
features: [class-static-fields-public, class-static-block]
---*/

var thrown = new Test262Error();
var caught;
var sameBlock = false;
var subsequentField = false;
var subsequentBlock = false;

try {
  class C {
    static {
      throw thrown;
      sameBlock = true;
    }
    static x = subsequentField = true;
    static {
      subsequentBlock = true;
    }
  }
} catch (error) {
  caught = error;
}

assert.sameValue(caught, thrown);
assert.sameValue(sameBlock, false, 'same block');
assert.sameValue(subsequentField, false, 'subsequent field');
assert.sameValue(subsequentBlock, false, 'subsequent block');
