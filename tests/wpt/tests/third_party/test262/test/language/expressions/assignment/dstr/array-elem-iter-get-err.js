// This file was procedurally generated from the following sources:
// - src/dstr-assignment/array-elem-iter-get-err.case
// - src/dstr-assignment/error/assignment-expr.template
/*---
description: Abrupt completion returned from GetIterator (AssignmentExpression)
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

    1. Let iterator be GetIterator(value).
    2. ReturnIfAbrupt(iterator).

---*/
var iterable = {};
iterable[Symbol.iterator] = function() {
  throw new Test262Error();
};
var _;

assert.throws(Test262Error, function() {
  0, [ _ ] = iterable;
});
