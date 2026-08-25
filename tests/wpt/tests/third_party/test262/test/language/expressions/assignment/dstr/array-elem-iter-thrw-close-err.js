// This file was procedurally generated from the following sources:
// - src/dstr-assignment/array-elem-iter-thrw-close-err.case
// - src/dstr-assignment/error/assignment-expr.template
/*---
description: IteratorClose is called when reference evaluation produces a "throw" completion (AssignmentExpression)
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

assert.throws(Test262Error, function() {
  0, [ {}[thrower()] ] = iterable;
});

assert.sameValue(nextCount, 0);
assert.sameValue(returnCount, 1);
