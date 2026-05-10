// Copyright (C) 2019 Caio Lima (Igalia SL). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: PrivateFieldGet should return with abrupt completion
esid: runtime-semantics-class-definition-evaluation
info: |
  ClassTail : ClassHeritage { ClassBody }
    ...
    28. For each ClassElement e in order from elements,
      a. If IsStatic of e is false, then
        i. Let field be the result of performing ClassElementEvaluation for e with arguments proto and false.
      b. Else,
        i. Let field be the result of performing PropertyDefinitionEvaluation for mClassElementEvaluation for e with arguments F and false.
      c. If field is an abrupt completion, then
        i. Set the running execution context's LexicalEnvironment to lex.
        ii. Set the running execution context's PrivateEnvironment to outerPrivateEnvironment.
        iii. Return Completion(field).
    ...
features: [class-fields-public, class-static-fields-public, class]
---*/

function abruptCompletion() {
  throw new Test262Error();
}

let neverExecuted = false;

assert.throws(Test262Error, function() {
  class C {
    [abruptCompletion()];
    [neverExecuted = true];
  }
}, 'computed property should have abrupt completion');
assert.sameValue(neverExecuted, false);

assert.throws(Test262Error, function() {
  class C {
    static [abruptCompletion()];
    [neverExecuted = true];
  }
}, 'static computed property should have abrupt completion');
assert.sameValue(neverExecuted, false);
