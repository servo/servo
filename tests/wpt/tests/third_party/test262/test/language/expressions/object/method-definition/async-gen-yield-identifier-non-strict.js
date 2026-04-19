// This file was procedurally generated from the following sources:
// - src/async-generators/yield-identifier-non-strict.case
// - src/async-generators/non-strict/async-obj-method.template
/*---
description: Use of yield as a valid identifier in a function body inside a generator body in non strict mode (Generator method - valid for non-strict only cases)
esid: prod-AsyncGeneratorMethod
features: [async-iteration]
flags: [generated, noStrict, async]
info: |
    Async Generator Function Definitions

    AsyncGeneratorMethod :
      async [no LineTerminator here] * PropertyName ( UniqueFormalParameters ) { AsyncGeneratorBody }

---*/


var callCount = 0;

var gen = {
  async *method() {
    callCount += 1;
    return (function(arg) {
        var yield = arg + 1;
        return yield;
      }(yield))
  }
}.method;

var iter = gen();

var item = iter.next();

item.then(({ done, value }) => {
  assert.sameValue(done, false);
  assert.sameValue(value, undefined);
});

item = iter.next(42);

item.then(({ done, value }) => {
  assert.sameValue(done, true);
  assert.sameValue(value, 43);
}).then($DONE, $DONE);

assert.sameValue(callCount, 1);
