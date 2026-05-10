// This file was procedurally generated from the following sources:
// - src/generators/yield-spread-arr-single.case
// - src/generators/default/expression.template
/*---
description: Use yield value in a array spread position (Unnamed generator expression)
esid: prod-GeneratorExpression
features: [generators]
flags: [generated]
info: |
    14.4 Generator Function Definitions

    GeneratorExpression:
      function * BindingIdentifier opt ( FormalParameters ) { GeneratorBody }


    Array Initializer

    SpreadElement[Yield, Await]:
      ...AssignmentExpression[+In, ?Yield, ?Await]
---*/
var arr = ['a', 'b', 'c'];

var callCount = 0;

var gen = function *() {
  callCount += 1;
  yield [...yield];
};

var iter = gen();

iter.next(false);
var item = iter.next(arr);
var value = item.value;

assert.notSameValue(value, arr, 'value is a new array');
assert(Array.isArray(value), 'value is an Array exotic object');
assert.sameValue(value.length, 3)
assert.sameValue(value[0], 'a');
assert.sameValue(value[1], 'b');
assert.sameValue(value[2], 'c');
assert.sameValue(item.done, false);

assert.sameValue(callCount, 1);
