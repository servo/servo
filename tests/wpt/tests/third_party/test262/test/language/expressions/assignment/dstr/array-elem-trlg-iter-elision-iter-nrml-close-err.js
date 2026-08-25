// This file was procedurally generated from the following sources:
// - src/dstr-assignment/array-elem-trlg-iter-elision-iter-nrml-close-err.case
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

    ArrayAssignmentPattern :
        [ AssignmentElementList , Elisionopt AssignmentRestElementopt ]

    [...]
    6. If Elision is present, then
       a. Let status be the result of performing
          IteratorDestructuringAssignmentEvaluation of Elision with
          iteratorRecord as the argument.
       b. If status is an abrupt completion, then
          [...]
    8. If iteratorRecord.[[done]] is false, return IteratorClose(iterator,
       status).

---*/
var nextCount = 0;
var returnCount = 0;
var iterable = {};
var x;
var iterator = {
  next: function() {
    nextCount += 1;

    // Set an upper-bound to limit unnecessary iteration in non-conformant
    // implementations
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

assert.throws(Test262Error, function() {
  0, [ x , , ] = iterable;
});

assert.sameValue(nextCount, 2);
assert.sameValue(returnCount, 1);
