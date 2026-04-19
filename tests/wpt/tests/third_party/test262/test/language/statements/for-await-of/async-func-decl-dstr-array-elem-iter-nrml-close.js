// This file was procedurally generated from the following sources:
// - src/dstr-assignment-for-await/array-elem-iter-nrml-close.case
// - src/dstr-assignment-for-await/default/async-func-decl.template
/*---
description: IteratorClose is called when assignment evaluation has not exhausted the iterator (for-await-of statement in an async function declaration)
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

    7.4.6 IteratorClose ( iterator, completion )

    [...]
    5. Let innerResult be Call(return, iterator, « »).
    [...]

---*/
let nextCount = 0;
let returnCount = 0;
let thisValue = null;
let args = null;
let _;
let iterable = {};
let iterator = {
  next() {
    nextCount += 1;
    // Set an upper-bound to limit unnecessary iteration in non-conformant
    // implementations
    return { done: nextCount > 10 };
  },
  return() {
    returnCount += 1;
    thisValue = this;
    args = arguments;
    return {};
  }
};
iterable[Symbol.iterator] = function() {
  return iterator;
};

let iterCount = 0;
async function fn() {
  for await ([ _ ] of [iterable]) {
    assert.sameValue(nextCount, 1);
    assert.sameValue(returnCount, 1);
    assert.sameValue(thisValue, iterator, 'correct `this` value');
    assert(!!args, 'arguments object provided');
    assert.sameValue(args.length, 0, 'zero arguments specified');

    iterCount += 1;
  }
}

let promise = fn();

promise
  .then(() => assert.sameValue(iterCount, 1, 'iteration occurred as expected'), $DONE)
  .then($DONE, $DONE);
