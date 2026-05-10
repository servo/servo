// This file was procedurally generated from the following sources:
// - src/generators/yield-identifier-non-strict.case
// - src/generators/non-strict/obj-method.template
/*---
description: Use of yield as a valid identifier in a function body inside a generator body in non strict mode (Generator method - valid for non-strict only cases)
esid: prod-GeneratorMethod
features: [generators]
flags: [generated, noStrict]
info: |
    14.4 Generator Function Definitions

    GeneratorMethod[Yield, Await]:
      * PropertyName ( UniqueFormalParameters ) { GeneratorBody }

---*/

var callCount = 0;

var gen = {
  *method() {
    callCount += 1;
    return (function(arg) {
        var yield = arg + 1;
        return yield;
      }(yield))
  }
}.method;

var iter = gen();

var item = iter.next();

assert.sameValue(item.done, false);
assert.sameValue(item.value, undefined);

item = iter.next(42);

assert.sameValue(item.done, true);
assert.sameValue(item.value, 43);

assert.sameValue(callCount, 1);
