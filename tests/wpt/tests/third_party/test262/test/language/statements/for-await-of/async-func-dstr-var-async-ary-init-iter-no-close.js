// This file was procedurally generated from the following sources:
// - src/dstr-binding-for-await/ary-init-iter-no-close.case
// - src/dstr-binding-for-await/default/for-await-of-async-func-var-async.template
/*---
description: Iterator is not closed when exhausted by pattern evaluation (for-await-of statement)
esid: sec-for-in-and-for-of-statements-runtime-semantics-labelledevaluation
features: [Symbol.iterator, destructuring-binding, async-iteration]
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

    13.3.3.5 Runtime Semantics: BindingInitialization

    BindingPattern : ArrayBindingPattern

    [...]
    4. If iteratorRecord.[[done]] is false, return ? IteratorClose(iterator,
       result).
    [...]

---*/
var doneCallCount = 0;
var iter = {};
iter[Symbol.iterator] = function() {
  return {
    next() {
      return { value: null, done: true };
    },
    return() {
      doneCallCount += 1;
      return {};
    }
  };
};

var iterCount = 0;
var asyncIter = (async function*() {
  yield* [iter];
})();

async function fn() {
  for await (var [x] of asyncIter) {
    assert.sameValue(doneCallCount, 0);

    iterCount += 1;
  }
}

fn()
  .then(() => assert.sameValue(iterCount, 1, 'iteration occurred as expected'), $DONE)
  .then($DONE, $DONE);
