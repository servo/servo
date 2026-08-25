// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-init-iter-no-close.case
// - src/dstr-binding/default/async-gen-meth.template
/*---
description: Iterator is not closed when exhausted by pattern evaluation (async generator method)
esid: sec-asyncgenerator-definitions-propertydefinitionevaluation
features: [Symbol.iterator, async-iteration]
flags: [generated, async]
info: |
    AsyncGeneratorMethod :
        async [no LineTerminator here] * PropertyName ( UniqueFormalParameters )
            { AsyncGeneratorBody }

    1. Let propKey be the result of evaluating PropertyName.
    2. ReturnIfAbrupt(propKey).
    3. If the function code for this AsyncGeneratorMethod is strict mode code, let strict be true.
       Otherwise let strict be false.
    4. Let scope be the running execution context's LexicalEnvironment.
    5. Let closure be ! AsyncGeneratorFunctionCreate(Method, UniqueFormalParameters,
       AsyncGeneratorBody, scope, strict).
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


var callCount = 0;
var obj = {
  async *method([x]) {
    assert.sameValue(doneCallCount, 0);
    callCount = callCount + 1;
  }
};

obj.method(iter).next().then(() => {
    assert.sameValue(callCount, 1, 'invoked exactly once');
}).then($DONE, $DONE);
