// This file was procedurally generated from the following sources:
// - src/generators/yield-spread-obj.case
// - src/generators/default/class-expr-static-method.template
/*---
description: Use yield value in a object spread position (Static generator method as a ClassExpression element)
esid: prod-GeneratorMethod
features: [object-spread, generators]
flags: [generated]
info: |
    ClassElement :
      static MethodDefinition

    MethodDefinition :
      GeneratorMethod

    14.4 Generator Function Definitions

    GeneratorMethod :
      * PropertyName ( UniqueFormalParameters ) { GeneratorBody }


    Spread Properties

    PropertyDefinition[Yield]:
      (...)
      ...AssignmentExpression[In, ?Yield]

---*/

var callCount = 0;

var C = class { static *gen() {
    callCount += 1;
    yield {
        ...yield,
        y: 1,
        ...yield yield,
      };
}}

var gen = C.gen;

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
