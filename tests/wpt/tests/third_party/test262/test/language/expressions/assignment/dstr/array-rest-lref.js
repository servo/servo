// This file was procedurally generated from the following sources:
// - src/dstr-assignment/array-rest-lref.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: Reference is evaluated during assignment (AssignmentExpression)
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

    ArrayAssignmentPattern : [ Elisionopt AssignmentRestElement ]

    [...]
    5. Let result be the result of performing
       IteratorDestructuringAssignmentEvaluation of AssignmentRestElement with
       iteratorRecord as the argument
    6. If iteratorRecord.[[done]] is false, return IteratorClose(iterator,
       result).

    AssignmentRestElement[Yield] : ... DestructuringAssignmentTarget

    1. If DestructuringAssignmentTarget is neither an ObjectLiteral nor an
       ArrayLiteral, then
       a. Let lref be the result of evaluating DestructuringAssignmentTarget.
       b. ReturnIfAbrupt(lref).
    [...]

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
  }
};
var obj = {};
iterable[Symbol.iterator] = function() {
  return iterator;
};

var result;
var vals = iterable;

result = [...obj['a' + 'b']] = vals;

assert.sameValue(nextCount, 1);
assert.sameValue(returnCount, 0);
assert(!!obj.ab);
assert.sameValue(obj.ab.length, 0);

assert.sameValue(result, vals);
