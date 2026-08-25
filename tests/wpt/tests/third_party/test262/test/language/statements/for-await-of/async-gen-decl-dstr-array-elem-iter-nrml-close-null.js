// This file was procedurally generated from the following sources:
// - src/dstr-assignment-for-await/array-elem-iter-nrml-close-null.case
// - src/dstr-assignment-for-await/async-generator/async-gen-decl.template
/*---
description: IteratorClose throws a TypeError when `return` returns a non-Object value (for-await-of statement in an async generator declaration)
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

    7.4.6 IteratorClose( iterator, completion )

    [...]
    5. Let innerResult be Call(return, iterator, « »).
    6. If completion.[[Type]] is throw, return Completion(completion).
    7. If innerResult.[[Type]] is throw, return Completion(innerResult).
    8. If Type(innerResult.[[Value]]) is not Object, throw a TypeError exception.

---*/
let _;
let nextCount = 0;
let iterator = {
  next() {
    nextCount += 1;
    // Set an upper-bound to limit unnecessary iteration in non-conformant
    // implementations
    return { done: nextCount > 10 };
  },
  return() {
    return null;
  }
};
let iterable = {
  [Symbol.iterator]() {
    return iterator;
  }
};

let iterCount = 0;
async function * fn() {
  for await ([ _ ] of [iterable]) {
    
    iterCount += 1;
  }
}

let iter = fn();

iter.next().then(() => $DONE('Promise incorrectly fulfilled.'), ({ constructor }) => {
  assert.sameValue(nextCount, 1);
  assert.sameValue(constructor, TypeError);
}).then($DONE, $DONE);

