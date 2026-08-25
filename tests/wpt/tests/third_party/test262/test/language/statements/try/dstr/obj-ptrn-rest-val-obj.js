// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-rest-val-obj.case
// - src/dstr-binding/default/try.template
/*---
description: Rest object contains just unextracted data (try statement)
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

var ranCatch = false;

try {
  throw {x: 1, y: 2, a: 5, b: 3};
} catch ({a, b, ...rest}) {
  assert.sameValue(rest.a, undefined);
  assert.sameValue(rest.b, undefined);

  verifyProperty(rest, "x", {
    enumerable: true,
    writable: true,
    configurable: true,
    value: 1
  });

  verifyProperty(rest, "y", {
    enumerable: true,
    writable: true,
    configurable: true,
    value: 2
  });
  ranCatch = true;
}

assert(ranCatch, 'executed `catch` block');
