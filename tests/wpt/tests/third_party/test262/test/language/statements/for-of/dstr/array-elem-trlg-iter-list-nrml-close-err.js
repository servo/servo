// This file was procedurally generated from the following sources:
// - src/dstr-assignment/array-elem-trlg-iter-list-nrml-close-err.case
// - src/dstr-assignment/error/for-of.template
/*---
description: Abrupt completion returned from IteratorClose (For..of statement)
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

    ArrayAssignmentPattern :
        [ AssignmentElementList , Elisionopt AssignmentRestElementopt ]

    [...]
    3. Let iteratorRecord be Record {[[iterator]]: iterator, [[done]]: false}.
    4. Let status be the result of performing
       IteratorDestructuringAssignmentEvaluation of AssignmentElementList using
       iteratorRecord as the argument.
    5. If status is an abrupt completion, then
       a. If iteratorRecord.[[done]] is false, return IteratorClose(iterator,
          status).
       b. Return Completion(status).

---*/
var nextCount = 0;
var returnCount = 0;
var iterable = {};
var thrower = function() {
  throw new Test262Error();
};
var x;
var iterator = {
  next: function() {
    nextCount += 1;
    return { done: nextCount > 10 };
  },
  return: function() {
    returnCount += 1;
    throw new Test262Error();
  }
};
iterable[Symbol.iterator] = function() {
  return iterator;
};

var counter = 0;

assert.throws(Test262Error, function() {
  for ([ x , ] of [iterable]) {
    counter += 1;
  }
  counter += 1;
});

assert.sameValue(counter, 0);

assert.sameValue(nextCount, 1);
assert.sameValue(returnCount, 1);
