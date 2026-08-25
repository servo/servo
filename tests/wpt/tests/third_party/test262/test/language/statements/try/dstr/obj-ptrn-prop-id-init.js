// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-prop-id-init.case
// - src/dstr-binding/default/try.template
/*---
description: Binding as specified via property name, identifier, and initializer (try statement)
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
    8. Return InitializeReferencedBinding(lhs, v).
---*/

var ranCatch = false;

try {
  throw { };
} catch ({ x: y = 33 }) {
  assert.sameValue(y, 33);
  assert.throws(ReferenceError, function() {
    x;
  });
  ranCatch = true;
}

assert(ranCatch, 'executed `catch` block');
