// This file was procedurally generated from the following sources:
// - src/dstr-assignment/array-elem-trlg-iter-list-thrw-close-err.case
// - src/dstr-assignment/error/assignment-expr.template
/*---
description: IteratorClose is invoked when evaluation of AssignmentElementList returns a "throw" completion and the iterator has not been marked as "done" (AssignmentExpression)
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

    7.4.6 IteratorClose( iterator, completion )

    [...]
    7. If completion.[[type]] is throw, return Completion(completion).

---*/
var nextCount = 0;
var returnCount = 0;
var iterable = {};
var thrower = function() {
  throw new Test262Error();
};
function ReturnError() {}
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
iterable[Symbol.iterator] = function() {
  return iterator;
};


assert.throws(Test262Error, function() {
  0, [ {}[thrower()] , ] = iterable;
});


assert.sameValue(nextCount, 0);
assert.sameValue(returnCount, 1);
