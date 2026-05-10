// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-id-init-skipped.case
// - src/dstr-binding/default/try.template
/*---
description: Destructuring initializer is not evaluated when value is not `undefined` (try statement)
esid: sec-runtime-semantics-catchclauseevaluation
features: [destructuring-binding]
flags: [generated]
info: |
    Catch : catch ( CatchParameter ) Block

    [...]
    5. Let status be the result of performing BindingInitialization for
       CatchParameter passing thrownValue and catchEnv as arguments.
    [...]

    13.3.3.7 Runtime Semantics: KeyedBindingInitialization

    SingleNameBinding : BindingIdentifier Initializeropt

    [...]
    6. If Initializer is present and v is undefined, then
       [...]
    [...]
---*/
var initCount = 0;
function counter() {
  initCount += 1;
}

var ranCatch = false;

try {
  throw { w: null, x: 0, y: false, z: '' };
} catch ({ w = counter(), x = counter(), y = counter(), z = counter() }) {
  assert.sameValue(w, null);
  assert.sameValue(x, 0);
  assert.sameValue(y, false);
  assert.sameValue(z, '');
  assert.sameValue(initCount, 0);
  ranCatch = true;
}

assert(ranCatch, 'executed `catch` block');
