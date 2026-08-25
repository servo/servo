// This file was procedurally generated from the following sources:
// - src/dstr-assignment/array-elem-trlg-iter-list-thrw-close-skip.case
// - src/dstr-assignment/error/assignment-expr.template
/*---
description: IteratorClose is not invoked when evaluation of AssignmentElementList returns an abrupt completion and the iterator has been marked as "done" (AssignmentExpression)
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
var x;

assert.throws(Test262Error, function() {
  0, [ x , ] = iterable;
});

assert.sameValue(nextCount, 1);
assert.sameValue(returnCount, 0);
