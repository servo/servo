// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-rest-id-elision-next-err.case
// - src/dstr-binding/error/try.template
/*---
description: Rest element following elision elements (try statement)
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
    ArrayBindingPattern : [ Elisionopt BindingRestElement ]
    1. If Elision is present, then
       a. Let status be the result of performing
          IteratorDestructuringAssignmentEvaluation of Elision with
          iteratorRecord as the argument.
       b. ReturnIfAbrupt(status).
    2. Return the result of performing IteratorBindingInitialization for
       BindingRestElement with iteratorRecord and environment as arguments.

---*/
var iter = (function*() { throw new Test262Error(); })();

assert.throws(Test262Error, function() {
  try {
    throw iter;
  } catch ([, ...x]) {}
});
