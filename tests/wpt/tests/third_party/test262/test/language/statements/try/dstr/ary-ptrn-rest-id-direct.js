// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-rest-id-direct.case
// - src/dstr-binding/default/try.template
/*---
description: Lone rest element (direct binding) (try statement)
esid: sec-runtime-semantics-catchclauseevaluation
features: [destructuring-binding]
flags: [generated]
includes: [compareArray.js]
info: |
    Catch : catch ( CatchParameter ) Block

    [...]
    5. Let status be the result of performing BindingInitialization for
       CatchParameter passing thrownValue and catchEnv as arguments.
    [...]

    Runtime Semantics: IteratorBindingInitialization

    BindingRestElement : ... BindingIdentifier

    [...]
    2. Let A be ! ArrayCreate(0).
    3. Let n be 0.
    4. Repeat,
        [...]
        f. Perform ! CreateDataPropertyOrThrow(A, ! ToString(n), nextValue).
        g. Set n to n + 1.

---*/

var ranCatch = false;

try {
  throw [1];
} catch ([...x]) {
  assert(Array.isArray(x));
  assert.compareArray(x, [1]);
  ranCatch = true;
}

assert(ranCatch, 'executed `catch` block');
