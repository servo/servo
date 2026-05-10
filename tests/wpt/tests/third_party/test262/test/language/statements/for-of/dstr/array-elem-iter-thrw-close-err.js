// This file was procedurally generated from the following sources:
// - src/dstr-assignment/array-elem-iter-thrw-close-err.case
// - src/dstr-assignment/error/for-of.template
/*---
description: IteratorClose is called when reference evaluation produces a "throw" completion (For..of statement)
esid: sec-for-in-and-for-of-statements-runtime-semantics-labelledevaluation
features: [Symbol.iterator, destructuring-binding]
flags: [generated]
info: |
    IterationStatement :
      for ( LeftHandSideExpression of AssignmentExpression ) Statement

    1. Let keyResult be the result of performing ? ForIn/OfHeadEvaluation(« »,
       AssignmentExpression, iterate).
    2. Return ? ForIn/OfBodyEvaluation(LeftHandSideExpression, Statement,
       keyResult, assignment, labelSet).

    13.7.5.13 Runtime Semantics: ForIn/OfBodyEvaluation

    [...]
    4. If destructuring is true and if lhsKind is assignment, then
       a. Assert: lhs is a LeftHandSideExpression.
       b. Let assignmentPattern be the parse of the source text corresponding to
          lhs using AssignmentPattern as the goal symbol.
    [...]

    ArrayAssignmentPattern : [ AssignmentElementList ]

    [...]
    5. If iteratorRecord.[[done]] is false, return IteratorClose(iterator,
       result).
    6. Return result.

---*/
var nextCount = 0;
var returnCount = 0;
function ReturnError() {}
var iterable = {};
var iterator = {
  next: function() {
    nextCount += 1;
    return { done: true };
  },
  return: function() {
    returnCount += 1;

    // This value should be discarded.
    throw new ReturnError();
  }
};
var thrower = function() {
  throw new Test262Error();
};
iterable[Symbol.iterator] = function() {
  return iterator;
};

var counter = 0;

assert.throws(Test262Error, function() {
  for ([ {}[thrower()] ] of [iterable]) {
    counter += 1;
  }
  counter += 1;
});

assert.sameValue(counter, 0);

assert.sameValue(nextCount, 0);
assert.sameValue(returnCount, 1);
