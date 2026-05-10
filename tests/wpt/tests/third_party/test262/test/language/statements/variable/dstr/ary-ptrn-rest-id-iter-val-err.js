// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-rest-id-iter-val-err.case
// - src/dstr-binding/error/var-stmt.template
/*---
description: Error forwarding when IteratorValue returns an abrupt completion (`var` statement)
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

    13.3.3.6 Runtime Semantics: IteratorBindingInitialization
    BindingRestElement : ... BindingIdentifier
    1. Let lhs be ResolveBinding(StringValue of BindingIdentifier,
       environment).
    2. ReturnIfAbrupt(lhs). 3. Let A be ArrayCreate(0). 4. Let n=0. 5. Repeat,
       [...]
       c. Let nextValue be IteratorValue(next).
       d. If nextValue is an abrupt completion, set iteratorRecord.[[done]] to
          true.
       e. ReturnIfAbrupt(nextValue).

---*/
var poisonedValue = Object.defineProperty({}, 'value', {
  get: function() {
    throw new Test262Error();
  }
});
var iter = {};
iter[Symbol.iterator] = function() {
  return {
    next: function() {
      return poisonedValue;
    }
  };
};

assert.throws(Test262Error, function() {
  var [...x] = iter;
});
