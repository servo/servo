// This file was procedurally generated from the following sources:
// - src/dstr-assignment/array-rest-put-prop-ref-user-err-iter-close-skip.case
// - src/dstr-assignment/error/for-of.template
/*---
description: IteratorClose is not called when value assignment produces an abrupt completion. (For..of statement)
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

    ArrayAssignmentPattern : [ Elisionopt AssignmentRestElement ]

    [...]
    5. Let result be the result of performing
       IteratorDestructuringAssignmentEvaluation of AssignmentRestElement with
       iteratorRecord as the argument
    6. If iteratorRecord.[[done]] is false, return IteratorClose(iterator,
       result).

    AssignmentRestElement[Yield] : ... DestructuringAssignmentTarget

    [...]
    4. Repeat while iteratorRecord.[[done]] is false
       [...]
       d. If next is false, set iteratorRecord.[[done]] to true.
       [...]
    5. If DestructuringAssignmentTarget is neither an ObjectLiteral nor an
       ArrayLiteral, then
       a. Return PutValue(lref, A).

---*/
var nextCount = 0;
var returnCount = 0;
var iterable = {};
var iterator = {
  next: function() {
    nextCount += 1;
    return { done: true };
  },
  return: function() {
    returnCount += 1;
  }
};
var obj = Object.defineProperty({}, 'poisoned', {
  set: function(x) {
    throw new Test262Error();
  }
});
iterable[Symbol.iterator] = function() {
  return iterator;
};

var counter = 0;

assert.throws(Test262Error, function() {
  for ([...obj.poisoned] of [iterable]) {
    counter += 1;
  }
  counter += 1;
});

assert.sameValue(counter, 0);

assert.sameValue(nextCount, 1);
assert.sameValue(returnCount, 0);
