// This file was procedurally generated from the following sources:
// - src/generators/yield-identifier-non-strict.case
// - src/generators/non-strict/declaration.template
/*---
description: Use of yield as a valid identifier in a function body inside a generator body in non strict mode (Generator function declaration - valid for non-strict only cases)
esid: prod-GeneratorDeclaration
features: [generators]
flags: [generated, noStrict]
info: |
    14.4 Generator Function Definitions

    GeneratorDeclaration[Yield, Await, Default]:
      function * BindingIdentifier[?Yield, ?Await] ( FormalParameters[+Yield, ~Await] ) { GeneratorBody }

---*/

var callCount = 0;

function *gen() {
  callCount += 1;
  return (function(arg) {
      var yield = arg + 1;
      return yield;
    }(yield))
}

var iter = gen();

var item = iter.next();

assert.sameValue(item.done, false);
assert.sameValue(item.value, undefined);

item = iter.next(42);

assert.sameValue(item.done, true);
assert.sameValue(item.value, 43);

assert.sameValue(callCount, 1);
