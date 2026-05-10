// This file was procedurally generated from the following sources:
// - src/dstr-assignment/array-rest-iter-thrw-close-skip.case
// - src/dstr-assignment/error/assignment-expr.template
/*---
description: IteratorClose is not called when iteration produces an abrupt completion (AssignmentExpression)
esid: sec-variable-statement-runtime-semantics-evaluation
features: [Symbol.iterator, destructuring-binding]
flags: [generated]
info: |
    VariableDeclaration : BindingPattern Initializer

    1. Let rhs be the result of evaluating Initializer.
    2. Let rval be GetValue(rhs).
    3. ReturnIfAbrupt(rval).
    4. Return the result of performing BindingInitialization for
       BindingPattern passing rval and undefined as arguments.

    ArrayAssignmentPattern : [ Elisionopt AssignmentRestElement ]

    [...]
    5. Let result be the result of performing
       IteratorDestructuringAssignmentEvaluation of AssignmentRestElement with
       iteratorRecord as the argument
    6. If iteratorRecord.[[done]] is false, return IteratorClose(iterator,
       result).

---*/
var nextCount = 0;
var returnCount = 0;
var x;
var iterable = {};
var iterator = {
  next: function() {
    nextCount += 1;
    throw new Test262Error();
  },
  return: function() {
    returnCount += 1;
  }
};
iterable[Symbol.iterator] = function() {
  return iterator;
};

assert.throws(Test262Error, function() {
  0, [...x] = iterable;
});

assert.sameValue(nextCount, 1);
assert.sameValue(returnCount, 0);
