// This file was procedurally generated from the following sources:
// - src/generators/yield-spread-arr-multiple.case
// - src/generators/default/expression.template
/*---
description: Use yield value in a array spread position (Unnamed generator expression)
esid: prod-GeneratorExpression
features: [generators]
flags: [generated]
includes: [compareArray.js]
info: |
    14.4 Generator Function Definitions

    GeneratorExpression:
      function * BindingIdentifier opt ( FormalParameters ) { GeneratorBody }


    Array Initializer

    SpreadElement[Yield, Await]:
      ...AssignmentExpression[+In, ?Yield, ?Await]

---*/
var arr = ['a', 'b', 'c'];
var item;

var callCount = 0;

var gen = function *() {
  callCount += 1;
  yield [...yield yield];
};

var iter = gen();

iter.next(false);
item = iter.next(['a', 'b', 'c']);
item = iter.next(item.value);

assert.compareArray(item.value, arr);
assert.sameValue(item.done, false);

assert.sameValue(callCount, 1);
