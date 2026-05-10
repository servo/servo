// This file was procedurally generated from the following sources:
// - src/dstr-assignment/array-elision-iter-nrml-close-null.case
// - src/dstr-assignment/error/assignment-expr.template
/*---
description: IteratorClose throws a TypeError when `return` returns a non-Object value (AssignmentExpression)
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

    ArrayAssignmentPattern : [ Elision ]

    1. Let iterator be GetIterator(value).
    [...]
    5. If iteratorRecord.[[done]] is false, return IteratorClose(iterator,
       result).
    [...]

    7.4.6 IteratorClose( iterator, completion )

    [...]
    6. Let innerResult be Call(return, iterator, « »).
    7. If completion.[[type]] is throw, return Completion(completion).
    8. If innerResult.[[type]] is throw, return Completion(innerResult).
    9. If Type(innerResult.[[value]]) is not Object, throw a TypeError
       exception.

---*/
var iterable = {};
var nextCount = 0;
var iterator = {
  next: function() {
    nextCount += 1;
    // Set an upper-bound to limit unnecessary iteration in non-conformant
    // implementations
    return { done: nextCount > 10 };
  },
  return: function() {
    return null;
  }
};
iterable[Symbol.iterator] = function() {
  return iterator;
};

assert.throws(TypeError, function() {
  0, [ , ] = iterable;
});
