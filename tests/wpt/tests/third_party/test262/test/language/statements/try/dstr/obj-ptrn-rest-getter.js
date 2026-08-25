// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-rest-getter.case
// - src/dstr-binding/default/try.template
/*---
description: Getter is called when obj is being deconstructed to a rest Object (try statement)
esid: sec-runtime-semantics-catchclauseevaluation
features: [object-rest, destructuring-binding]
flags: [generated]
includes: [propertyHelper.js]
info: |
    Catch : catch ( CatchParameter ) Block

    [...]
    5. Let status be the result of performing BindingInitialization for
       CatchParameter passing thrownValue and catchEnv as arguments.
    [...]
---*/
var count = 0;

var ranCatch = false;

try {
  throw { get v() { count++; return 2; } };
} catch ({...x}) {
  assert.sameValue(count, 1);

  verifyProperty(x, "v", {
    enumerable: true,
    writable: true,
    configurable: true,
    value: 2
  });
  ranCatch = true;
}

assert(ranCatch, 'executed `catch` block');
