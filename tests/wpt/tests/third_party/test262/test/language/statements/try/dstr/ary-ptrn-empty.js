// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-empty.case
// - src/dstr-binding/default/try.template
/*---
description: No iteration occurs for an "empty" array binding pattern (try statement)
esid: sec-runtime-semantics-catchclauseevaluation
features: [generators, destructuring-binding]
flags: [generated]
info: |
    Catch : catch ( CatchParameter ) Block

    [...]
    5. Let status be the result of performing BindingInitialization for
       CatchParameter passing thrownValue and catchEnv as arguments.
    [...]

    13.3.3.6 Runtime Semantics: IteratorBindingInitialization

    ArrayBindingPattern : [ ]

    1. Return NormalCompletion(empty).

---*/
var iterations = 0;
var iter = function*() {
  iterations += 1;
}();

var ranCatch = false;

try {
  throw iter;
} catch ([]) {
  assert.sameValue(iterations, 0);
  ranCatch = true;
}

assert(ranCatch, 'executed `catch` block');
