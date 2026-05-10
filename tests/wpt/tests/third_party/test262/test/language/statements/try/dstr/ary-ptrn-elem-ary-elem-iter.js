// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-elem-ary-elem-iter.case
// - src/dstr-binding/default/try.template
/*---
description: BindingElement with array binding pattern and initializer is not used (try statement)
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

    BindingElement : BindingPatternInitializer opt

    1. If iteratorRecord.[[done]] is false, then
       a. Let next be IteratorStep(iteratorRecord.[[iterator]]).
       [...]
       e. Else,
          i. Let v be IteratorValue(next).
          [...]
    4. Return the result of performing BindingInitialization of BindingPattern
       with v and environment as the arguments.
---*/

var ranCatch = false;

try {
  throw [[7, 8, 9]];
} catch ([[x, y, z] = [4, 5, 6]]) {
  assert.sameValue(x, 7);
  assert.sameValue(y, 8);
  assert.sameValue(z, 9);
  ranCatch = true;
}

assert(ranCatch, 'executed `catch` block');
