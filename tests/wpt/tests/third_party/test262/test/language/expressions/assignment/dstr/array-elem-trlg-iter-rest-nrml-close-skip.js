// This file was procedurally generated from the following sources:
// - src/dstr-assignment/array-elem-trlg-iter-rest-nrml-close-skip.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: IteratorClose is not called when rest element evaluation has exhausted the iterator (AssignmentExpression)
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
    7. If AssignmentRestElement is present, then
       a. Let status be the result of performing
          IteratorDestructuringAssignmentEvaluation of AssignmentRestElement
          with iteratorRecord as the argument.
    8. If iteratorRecord.[[done]] is false, return IteratorClose(iterator,
       status).
    9. Return Completion(status).

---*/
var nextCount = 0;
var returnCount = 0;
var iterable = {};
var x, y;
var iterator = {
  next: function() {
    nextCount += 1;
    return { value: nextCount, done: nextCount > 1 };
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

result = [ x , ...y ] = vals;



assert.sameValue(result, vals);

assert.sameValue(nextCount, 2, 'nextCount');
assert.sameValue(returnCount, 0, 'returnCount');
assert.sameValue(x, 1, 'x');
assert.sameValue(y.length, 0, 'y.length');
