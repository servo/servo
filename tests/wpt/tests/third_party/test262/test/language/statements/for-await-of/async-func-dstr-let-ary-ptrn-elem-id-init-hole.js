// This file was procedurally generated from the following sources:
// - src/dstr-binding-for-await/ary-ptrn-elem-id-init-hole.case
// - src/dstr-binding-for-await/default/for-await-of-async-func-let.template
/*---
description: Destructuring initializer with a "hole" (for-await-of statement)
esid: sec-for-in-and-for-of-statements-runtime-semantics-labelledevaluation
features: [destructuring-binding, async-iteration]
flags: [generated, async]
info: |
    IterationStatement :
        for await ( ForDeclaration of AssignmentExpression ) Statement

    [...]
    2. Return ? ForIn/OfBodyEvaluation(ForDeclaration, Statement, keyResult,
        lexicalBinding, labelSet, async).

    13.7.5.13 Runtime Semantics: ForIn/OfBodyEvaluation

    [...]
    4. Let destructuring be IsDestructuring of lhs.
    [...]
    6. Repeat
       [...]
       j. If destructuring is false, then
          [...]
       k. Else
          i. If lhsKind is assignment, then
             [...]
          ii. Else if lhsKind is varBinding, then
              [...]
          iii. Else,
               1. Assert: lhsKind is lexicalBinding.
               2. Assert: lhs is a ForDeclaration.
               3. Let status be the result of performing BindingInitialization
                  for lhs passing nextValue and iterationEnv as arguments.
          [...]

    13.3.3.6 Runtime Semantics: IteratorBindingInitialization
    SingleNameBinding : BindingIdentifier Initializer_opt
    [...] 5. If Initializer is present and v is undefined, then
       a. Let defaultValue be the result of evaluating Initializer.
       b. Let v be GetValue(defaultValue).
       [...]
    6. If environment is undefined, return PutValue(lhs, v). 7. Return InitializeReferencedBinding(lhs, v).
---*/

var iterCount = 0;

async function fn() {
  for await (let [x = 23] of [[,]]) {
    assert.sameValue(x, 23);
    // another statement

    iterCount += 1;
  }
}

fn()
  .then(() => assert.sameValue(iterCount, 1, 'iteration occurred as expected'), $DONE)
  .then($DONE, $DONE);

