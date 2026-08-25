// This file was procedurally generated from the following sources:
// - src/generators/yield-spread-obj.case
// - src/generators/default/expression.template
/*---
description: Use yield value in a object spread position (Unnamed generator expression)
esid: prod-GeneratorExpression
features: [object-spread, generators]
flags: [generated]
info: |
    14.4 Generator Function Definitions

    GeneratorExpression:
      function * BindingIdentifier opt ( FormalParameters ) { GeneratorBody }


    Spread Properties

    PropertyDefinition[Yield]:
      (...)
      ...AssignmentExpression[In, ?Yield]

---*/

var callCount = 0;

var gen = function *() {
  callCount += 1;
  yield {
      ...yield,
      y: 1,
      ...yield yield,
    };
};

var iter = gen();

iter.next();
iter.next({ x: 42 });
iter.next({ x: 'lol' });
var item = iter.next({ y: 39 });

assert.sameValue(item.value.x, 42);
assert.sameValue(item.value.y, 39);
assert.sameValue(Object.keys(item.value).length, 2);
assert.sameValue(item.done, false);

assert.sameValue(callCount, 1);
