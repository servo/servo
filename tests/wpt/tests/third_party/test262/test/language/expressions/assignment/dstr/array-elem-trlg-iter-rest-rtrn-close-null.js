// This file was procedurally generated from the following sources:
// - src/dstr-assignment/array-elem-trlg-iter-rest-rtrn-close-null.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: IteratorClose throws a TypeError when `return` returns a non-Object value (AssignmentExpression)
esid: sec-variable-statement-runtime-semantics-evaluation
features: [Symbol.iterator, generators, destructuring-binding]
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

    7.4.6 IteratorClose( iterator, completion )

    [...]
    6. Let innerResult be Call(return, iterator, « »).
    7. If completion.[[type]] is throw, return Completion(completion).
    8. If innerResult.[[type]] is throw, return Completion(innerResult).

---*/
var nextCount = 0;
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
    return null;
  }
};
var iter;
iterable[Symbol.iterator] = function() {
  return iterator;
};

function* g() {

var result;
var vals = iterable;

result = [ x , ...{}[yield] ] = vals;



assert.sameValue(result, vals);

}

iter = g();
iter.next();

assert.throws(TypeError, function() {
  iter.return();
});

assert.sameValue(nextCount, 1);
