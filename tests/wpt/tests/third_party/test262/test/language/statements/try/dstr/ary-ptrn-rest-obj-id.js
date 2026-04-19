// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-rest-obj-id.case
// - src/dstr-binding/default/try.template
/*---
description: Rest element containing an object binding pattern (try statement)
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

    BindingRestElement : ... BindingPattern

    1. Let A be ArrayCreate(0).
    [...]
    3. Repeat
       [...]
       b. If iteratorRecord.[[done]] is true, then
          i. Return the result of performing BindingInitialization of
             BindingPattern with A and environment as the arguments.
       [...]
---*/

var ranCatch = false;

try {
  throw [1, 2, 3];
} catch ([...{ length }]) {
  assert.sameValue(length, 3);
  ranCatch = true;
}

assert(ranCatch, 'executed `catch` block');
