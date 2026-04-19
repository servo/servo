// This file was procedurally generated from the following sources:
// - src/dstr-assignment-for-await/array-elem-iter-get-err.case
// - src/dstr-assignment-for-await/async-generator/async-gen-decl.template
/*---
description: Abrupt completion returned from GetIterator (for-await-of statement in an async generator declaration)
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

    1. Let iterator be ? GetIterator(value).

---*/
let iterable = {
  [Symbol.iterator]() {
    throw new Test262Error();
  }
};
let _;


let iterCount = 0;
async function * fn() {
  for await ([ _ ] of [iterable]) {
    
    iterCount += 1;
  }
}

let iter = fn();

iter.next().then(() => $DONE('Promise incorrectly fulfilled.'), ({ constructor }) => {
  assert.sameValue(constructor, Test262Error);
}).then($DONE, $DONE);

