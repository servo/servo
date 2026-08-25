// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-rest-id-elision.case
// - src/dstr-binding/default/try.template
/*---
description: Rest element following elision elements (try statement)
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
    ArrayBindingPattern : [ Elisionopt BindingRestElement ]
    1. If Elision is present, then
       a. Let status be the result of performing
          IteratorDestructuringAssignmentEvaluation of Elision with
          iteratorRecord as the argument.
       b. ReturnIfAbrupt(status).
    2. Return the result of performing IteratorBindingInitialization for
       BindingRestElement with iteratorRecord and environment as arguments.
---*/
var values = [1, 2, 3, 4, 5];

var ranCatch = false;

try {
  throw values;
} catch ([ , , ...x]) {
  assert(Array.isArray(x));
  assert.sameValue(x.length, 3);
  assert.sameValue(x[0], 3);
  assert.sameValue(x[1], 4);
  assert.sameValue(x[2], 5);
  assert.notSameValue(x, values);
  ranCatch = true;
}

assert(ranCatch, 'executed `catch` block');
