// This file was procedurally generated from the following sources:
// - src/dstr-binding-for-await/obj-ptrn-prop-id-init.case
// - src/dstr-binding-for-await/default/for-await-of-async-func-const-async.template
/*---
description: Binding as specified via property name, identifier, and initializer (for-await-of statement)
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

    SingleNameBinding : BindingIdentifier Initializer_opt

    [...]
    7. Return InitializeReferencedBinding(lhs, v).
---*/

var iterCount = 0;
var asyncIter = (async function*() {
  yield* [{ }];
})();

async function fn() {
  for await (const { x: y = 33 } of asyncIter) {
    assert.sameValue(y, 33);
    assert.throws(ReferenceError, function() {
      x;
    });

    iterCount += 1;
  }
}

fn()
  .then(() => assert.sameValue(iterCount, 1, 'iteration occurred as expected'), $DONE)
  .then($DONE, $DONE);
