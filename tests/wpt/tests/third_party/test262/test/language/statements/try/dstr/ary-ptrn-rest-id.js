// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-rest-id.case
// - src/dstr-binding/default/try.template
/*---
description: Lone rest element (try statement)
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
    BindingRestElement : ... BindingIdentifier
    [...] 3. Let A be ArrayCreate(0). [...] 5. Repeat
       [...]
       f. Let status be CreateDataProperty(A, ToString (n), nextValue).
       [...]
---*/
var values = [1, 2, 3];

var ranCatch = false;

try {
  throw values;
} catch ([...x]) {
  assert(Array.isArray(x));
  assert.sameValue(x.length, 3);
  assert.sameValue(x[0], 1);
  assert.sameValue(x[1], 2);
  assert.sameValue(x[2], 3);
  assert.notSameValue(x, values);
  ranCatch = true;
}

assert(ranCatch, 'executed `catch` block');
