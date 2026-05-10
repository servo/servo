// This file was procedurally generated from the following sources:
// - src/dstr-binding-for-await/obj-ptrn-prop-ary-init.case
// - src/dstr-binding-for-await/default/for-await-of-async-gen-let-async.template
/*---
description: Object binding pattern with "nested" array binding pattern using initializer (for-await-of statement)
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

    13.3.3.7 Runtime Semantics: KeyedBindingInitialization

    [...]
    4. If Initializer is present and v is undefined, then
       a. Let defaultValue be the result of evaluating Initializer.
       b. Set v to ? GetValue(defaultValue).
    4. Return the result of performing BindingInitialization for BindingPattern
       passing v and environment as arguments.
---*/

var iterCount = 0;
var asyncIter = (async function*() {
  yield* [{}];
})();

async function *fn() {
  for await (let { w: [x, y, z] = [4, 5, 6] } of asyncIter) {
    assert.sameValue(x, 4);
    assert.sameValue(y, 5);
    assert.sameValue(z, 6);

    assert.throws(ReferenceError, function() {
      w;
    });

    iterCount += 1;
  }
}

fn().next()
  .then(() => assert.sameValue(iterCount, 1, 'iteration occurred as expected'), $DONE)
  .then($DONE, $DONE);
