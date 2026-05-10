// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-prop-ary.case
// - src/dstr-binding/default/try.template
/*---
description: Object binding pattern with "nested" array binding pattern not using initializer (try statement)
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

    [...]
    3. If Initializer is present and v is undefined, then
       [...]
    4. Return the result of performing BindingInitialization for BindingPattern
       passing v and environment as arguments.
---*/

var ranCatch = false;

try {
  throw { w: [7, undefined, ] };
} catch ({ w: [x, y, z] = [4, 5, 6] }) {
  assert.sameValue(x, 7);
  assert.sameValue(y, undefined);
  assert.sameValue(z, undefined);

  assert.throws(ReferenceError, function() {
    w;
  });
  ranCatch = true;
}

assert(ranCatch, 'executed `catch` block');
