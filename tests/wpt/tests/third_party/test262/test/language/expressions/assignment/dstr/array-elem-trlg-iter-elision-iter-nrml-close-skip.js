// This file was procedurally generated from the following sources:
// - src/dstr-assignment/array-elem-trlg-iter-elision-iter-nrml-close-skip.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: IteratorClose not invoked when elision exhausts the iterator (AssignmentExpression)
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
    9. Return Completion(status).

---*/
var nextCount = 0;
var returnCount = 0;
var iterable = {};
var x;
var iterator = {
  next: function() {
    nextCount += 1;

    return { done: nextCount > 1 };
  },
  return: function() {
    returnCount += 1;
  }
};
iterable[Symbol.iterator] = function() {
  return iterator;
};

var result;
var vals = iterable;

result = [ x , , ] = vals;

assert.sameValue(nextCount, 2);
assert.sameValue(returnCount, 0);

assert.sameValue(result, vals);
