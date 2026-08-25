// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-prop-eval-err.case
// - src/dstr-binding/error/try.template
/*---
description: Evaluation of property name returns an abrupt completion (try statement)
esid: sec-runtime-semantics-catchclauseevaluation
features: [destructuring-binding]
flags: [generated]
info: |
    Catch : catch ( CatchParameter ) Block

    [...]
    5. Let status be the result of performing BindingInitialization for
       CatchParameter passing thrownValue and catchEnv as arguments.
    [...]

    13.3.3.5 Runtime Semantics: BindingInitialization

    BindingProperty : PropertyName : BindingElement

    1. Let P be the result of evaluating PropertyName
    2. ReturnIfAbrupt(P).
---*/
function thrower() {
  throw new Test262Error();
}

assert.throws(Test262Error, function() {
  try {
    throw {};
  } catch ({ [thrower()]: x }) {}
});
