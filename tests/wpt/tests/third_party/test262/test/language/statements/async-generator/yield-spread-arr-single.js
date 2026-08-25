// This file was procedurally generated from the following sources:
// - src/async-generators/yield-spread-arr-single.case
// - src/async-generators/default/async-declaration.template
/*---
description: Use yield value in a array spread position (Async generator Function declaration)
esid: prod-AsyncGeneratorDeclaration
features: [async-iteration]
flags: [generated, async]
info: |
    Async Generator Function Definitions

    AsyncGeneratorDeclaration:
      async [no LineTerminator here] function * BindingIdentifier ( FormalParameters ) {
        AsyncGeneratorBody }


    Array Initializer

    SpreadElement[Yield, Await]:
      ...AssignmentExpression[+In, ?Yield, ?Await]

---*/
var arr = ['a', 'b', 'c'];


var callCount = 0;

async function *gen() {
  callCount += 1;
  yield [...yield];
}

var iter = gen();

iter.next(false);
var item = iter.next(arr);

item.then(({ done, value }) => {
  assert.notSameValue(value, arr, 'value is a new array');
  assert(Array.isArray(value), 'value is an Array exotic object');
  assert.sameValue(value.length, 3)
  assert.sameValue(value[0], 'a');
  assert.sameValue(value[1], 'b');
  assert.sameValue(value[2], 'c');
  assert.sameValue(done, false);
}).then($DONE, $DONE);

assert.sameValue(callCount, 1);
