// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-elem-id-init-throws.case
// - src/dstr-binding/error/try.template
/*---
description: Destructuring initializer returns an abrupt completion (try statement)
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
       c. ReturnIfAbrupt(v).
---*/

assert.throws(Test262Error, function() {
  try {
    throw [undefined];
  } catch ([x = (function() { throw new Test262Error(); })()]) {}
});
