// This file was procedurally generated from the following sources:
// - src/dstr-assignment-for-await/array-elem-iter-thrw-close-skip.case
// - src/dstr-assignment-for-await/async-generator/async-gen-decl.template
/*---
description: IteratorClose is not called when iteration produces an abrupt completion (for-await-of statement in an async generator declaration)
esid: sec-for-in-and-for-of-statements-runtime-semantics-labelledevaluation
features: [Symbol.iterator, destructuring-binding, async-iteration]
flags: [generated, async]
info: |
    IterationStatement :
      for await ( LeftHandSideExpression of AssignmentExpression ) Statement

    1. Let keyResult be the result of performing ? ForIn/OfHeadEvaluation(« »,
       AssignmentExpression, iterate).
    2. Return ? ForIn/OfBodyEvaluation(LeftHandSideExpression, Statement,
       keyResult, assignment, labelSet).

    13.7.5.13 Runtime Semantics: ForIn/OfBodyEvaluation

    [...]
    5. If destructuring is true and if lhsKind is assignment, then
       a. Assert: lhs is a LeftHandSideExpression.
       b. Let assignmentPattern be the parse of the source text corresponding to
          lhs using AssignmentPattern as the goal symbol.
    [...]

    ArrayAssignmentPattern : [ AssignmentElementList ]

    [...]
    4. If iteratorRecord.[[Done]] is false, return ? IteratorClose(iterator, result).
    5. Return result.

---*/
let nextCount = 0;
let returnCount = 0;
let iterator = {
  next() {
    nextCount += 1;
    throw new Test262Error();
  },
  return() {
    returnCount += 1;
  }
};
let iterable = {
  [Symbol.iterator]() {
    return iterator;
  }
};
let _;


let iterCount = 0;
async function * fn() {
  for await ([ x ] of [iterable]) {
    
    iterCount += 1;
  }
}

let iter = fn();

iter.next().then(() => $DONE('Promise incorrectly fulfilled.'), ({ constructor }) => {
  assert.sameValue(nextCount, 1);
  assert.sameValue(returnCount, 0);
}).then($DONE, $DONE);

