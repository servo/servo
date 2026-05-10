// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-init-iter-no-close.case
// - src/dstr-binding/default/try.template
/*---
description: Iterator is not closed when exhausted by pattern evaluation (try statement)
esid: sec-runtime-semantics-catchclauseevaluation
features: [Symbol.iterator, destructuring-binding]
flags: [generated]
info: |
    Catch : catch ( CatchParameter ) Block

    [...]
    5. Let status be the result of performing BindingInitialization for
       CatchParameter passing thrownValue and catchEnv as arguments.
    [...]

    13.3.3.5 Runtime Semantics: BindingInitialization

    BindingPattern : ArrayBindingPattern

    [...]
    4. If iteratorRecord.[[done]] is false, return ? IteratorClose(iterator,
       result).
    [...]

---*/
var doneCallCount = 0;
var iter = {};
iter[Symbol.iterator] = function() {
  return {
    next: function() {
      return { value: null, done: true };
    },
    return: function() {
      doneCallCount += 1;
      return {};
    }
  };
};

var ranCatch = false;

try {
  throw iter;
} catch ([x]) {
  assert.sameValue(doneCallCount, 0);
  ranCatch = true;
}

assert(ranCatch, 'executed `catch` block');
