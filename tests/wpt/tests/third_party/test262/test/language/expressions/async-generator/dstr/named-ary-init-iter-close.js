// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-init-iter-close.case
// - src/dstr-binding/default/async-gen-func-named-expr.template
/*---
description: Iterator is closed when not exhausted by pattern evaluation (async generator named function expression)
esid: sec-asyncgenerator-definitions-evaluation
features: [Symbol.iterator, async-iteration]
flags: [generated, async]
info: |
    AsyncGeneratorExpression : async [no LineTerminator here] function * BindingIdentifier
        ( FormalParameters ) { AsyncGeneratorBody }

        [...]
        7. Let closure be ! AsyncGeneratorFunctionCreate(Normal, FormalParameters,
           AsyncGeneratorBody, funcEnv, strict).
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
      return { value: null, done: false };
    },
    return: function() {
      doneCallCount += 1;
      return {};
    }
  };
};


var callCount = 0;
var f;
f = async function* h([x]) {
  assert.sameValue(doneCallCount, 1);
  callCount = callCount + 1;
};

f(iter).next().then(() => {
    assert.sameValue(callCount, 1, 'invoked exactly once');
}).then($DONE, $DONE);
