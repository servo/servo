// This file was procedurally generated from the following sources:
// - src/dstr-binding-for-await/ary-ptrn-elem-ary-empty-iter.case
// - src/dstr-binding-for-await/default/for-await-of-async-gen-let.template
/*---
description: BindingElement with array binding pattern and initializer is not used (for-await-of statement)
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

    BindingElement : BindingPattern Initializer_opt

    1. If iteratorRecord.[[done]] is false, then
       a. Let next be IteratorStep(iteratorRecord.[[iterator]]).
       [...]
       e. Else,
          i. Let v be IteratorValue(next).
          [...]
    4. Return the result of performing BindingInitialization of BindingPattern
       with v and environment as the arguments.
---*/
var initCount = 0;

var iterCount = 0;

async function *fn() {
  for await (let [[] = function() { initCount += 1; }()] of [[[23]]]) {
    assert.sameValue(initCount, 0);

    iterCount += 1;
  }
}

fn().next()
  .then(() => assert.sameValue(iterCount, 1, 'iteration occurred as expected'), $DONE)
  .then($DONE, $DONE);

