// This file was procedurally generated from the following sources:
// - src/async-generators/yield-identifier-spread-non-strict.case
// - src/async-generators/non-strict/async-obj-method.template
/*---
description: Mixed use of object spread and yield as a valid identifier in a function body inside a generator body in non strict mode (Generator method - valid for non-strict only cases)
esid: prod-AsyncGeneratorMethod
features: [object-spread, Symbol, async-iteration]
flags: [generated, noStrict, async]
info: |
    Async Generator Function Definitions

    AsyncGeneratorMethod :
      async [no LineTerminator here] * PropertyName ( UniqueFormalParameters ) { AsyncGeneratorBody }


    Spread Properties

    PropertyDefinition[Yield]:
      (...)
      ...AssignmentExpression[In, ?Yield]

---*/
var s = Symbol('s');


var callCount = 0;

var gen = {
  async *method() {
    callCount += 1;
    yield {
         ...yield yield,
         ...(function(arg) {
            var yield = arg;
            return {...yield};
         }(yield)),
         ...yield,
      }
  }
}.method;

var iter = gen();

var iter = gen();

iter.next();
iter.next();
iter.next({ x: 10, a: 0, b: 0, [s]: 1 });
iter.next({ y: 20, a: 1, b: 1, [s]: 42 });
var item = iter.next({ z: 30, b: 2 });

item.then(({ done, value }) => {
  assert.sameValue(done, false);
  assert.sameValue(value.x, 10);
  assert.sameValue(value.y, 20);
  assert.sameValue(value.z, 30);
  assert.sameValue(value.a, 1);
  assert.sameValue(value.b, 2);
  assert.sameValue(value[s], 42);
  assert.sameValue(Object.keys(value).length, 5);
  assert(Object.prototype.hasOwnProperty.call(value, s), "s is an own property");
}).then($DONE, $DONE);

assert.sameValue(callCount, 1);
