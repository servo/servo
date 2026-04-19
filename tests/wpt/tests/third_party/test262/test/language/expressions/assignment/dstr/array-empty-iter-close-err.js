// This file was procedurally generated from the following sources:
// - src/dstr-assignment/array-empty-iter-close-err.case
// - src/dstr-assignment/error/assignment-expr.template
/*---
description: Abrupt completion returned from IteratorClose (AssignmentExpression)
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

    ArrayAssignmentPattern : [ ]

    1. Let iterator be GetIterator(value).
    2. ReturnIfAbrupt(iterator).
    3. Return IteratorClose(iterator, NormalCompletion(empty)).

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
    throw new Test262Error();
  }
};
iterable[Symbol.iterator] = function() {
  return iterator;
};

assert.throws(Test262Error, function() {
  0, [] = iterable;
});

assert.sameValue(nextCount, 0);
assert.sameValue(returnCount, 1);
