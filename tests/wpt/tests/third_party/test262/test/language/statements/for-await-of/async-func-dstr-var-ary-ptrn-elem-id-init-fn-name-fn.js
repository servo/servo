// This file was procedurally generated from the following sources:
// - src/dstr-binding-for-await/ary-ptrn-elem-id-init-fn-name-fn.case
// - src/dstr-binding-for-await/default/for-await-of-async-func-var.template
/*---
description: SingleNameBinding assigns name to "anonymous" functions (for-await-of statement)
esid: sec-for-in-and-for-of-statements-runtime-semantics-labelledevaluation
features: [destructuring-binding, async-iteration]
flags: [generated, async]
info: |
    IterationStatement :
        for await ( var ForBinding of AssignmentExpression ) Statement

    [...]
    2. Return ? ForIn/OfBodyEvaluation(ForBinding, Statement, keyResult,
        varBinding, labelSet, async).

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
              1. Assert: lhs is a ForBinding.
              2. Let status be the result of performing BindingInitialization
                 for lhs passing nextValue and undefined as the arguments.
          [...]

    13.3.3.6 Runtime Semantics: IteratorBindingInitialization

    SingleNameBinding : BindingIdentifier Initializer_opt

    [...]
    5. If Initializer is present and v is undefined, then
       a. Let defaultValue be the result of evaluating Initializer.
       b. Set v to ? GetValue(defaultValue).
       c. If IsAnonymousFunctionDefinition(Initializer) is true, then
          [...]
    6. If environment is undefined, return PutValue(lhs, v).
    7. Return InitializeReferencedBinding(lhs, v).
---*/

var iterCount = 0;

async function fn() {
  for await (var [fn = function () {}, xFn = function x() {}] of [[]]) {
    assert.sameValue(fn.name, 'fn');
    assert.notSameValue(xFn.name, 'xFn');

    iterCount += 1;
  }
}

fn()
  .then(() => assert.sameValue(iterCount, 1, 'iteration occurred as expected'), $DONE)
  .then($DONE, $DONE);

