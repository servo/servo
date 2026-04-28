// Copyright (C) 2017 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Class definition should error if evaluation of static ClassElementName errors
esid: runtime-semantics-class-definition-evaluation
info: |
  Runtime Semantics: ClassDefinitionEvaluation
  ...
  27. For each ClassElement e in order from elements
    a. If IsStatic of e is false, then
      i. Let fields be the result of performing ClassElementEvaluation
          for e with arguments proto and false.
    b. Else,
      i. Let fields be the result of performing ClassElementEvaluation
          for e with arguments F and false.
    c. If fields is an abrupt completion, then
      i. Set the running execution context's LexicalEnvironment to lex.
      ii. Set the running execution context's PrivateNameEnvironment to outerPrivateEnvironment.
      iii. Return Completion(status).

features: [class-static-fields-public]
---*/

function f() {
  throw new Test262Error();
}

assert.throws(Test262Error, function() {
  class C {
    static [f()]
  }
});
