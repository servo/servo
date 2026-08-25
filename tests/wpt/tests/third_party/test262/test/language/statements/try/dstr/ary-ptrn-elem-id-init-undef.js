// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-elem-id-init-undef.case
// - src/dstr-binding/default/try.template
/*---
description: Destructuring initializer with an undefined value (try statement)
esid: sec-runtime-semantics-catchclauseevaluation
features: [destructuring-binding]
flags: [generated]
info: |
    Catch : catch ( CatchParameter ) Block

    [...]
    5. Let status be the result of performing BindingInitialization for
       CatchParameter passing thrownValue and catchEnv as arguments.
    [...]

    13.3.3.6 Runtime Semantics: IteratorBindingInitialization

    SingleNameBinding : BindingIdentifier Initializeropt

    [...]
    6. If Initializer is present and v is undefined, then
       a. Let defaultValue be the result of evaluating Initializer.
       b. Let v be GetValue(defaultValue).
       [...]
    7. If environment is undefined, return PutValue(lhs, v).
    8. Return InitializeReferencedBinding(lhs, v).
---*/

var ranCatch = false;

try {
  throw [undefined];
} catch ([x = 23]) {
  assert.sameValue(x, 23);
  ranCatch = true;
}

assert(ranCatch, 'executed `catch` block');
