// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-init-iter-close.case
// - src/dstr-binding/default/async-gen-func-decl-dflt.template
/*---
description: Iterator is closed when not exhausted by pattern evaluation (async generator function declaration (default parameter))
esid: sec-asyncgenerator-definitions-instantiatefunctionobject
features: [Symbol.iterator, async-iteration]
flags: [generated, async]
info: |
    AsyncGeneratorDeclaration : async [no LineTerminator here] function * BindingIdentifier
        ( FormalParameters ) { AsyncGeneratorBody }

        [...]
        3. Let F be ! AsyncGeneratorFunctionCreate(Normal, FormalParameters, AsyncGeneratorBody,
            scope, strict).
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
async function* f([x] = iter) {
  assert.sameValue(doneCallCount, 1);
  callCount = callCount + 1;
};
f().next().then(() => {
    assert.sameValue(callCount, 1, 'invoked exactly once');
}).then($DONE, $DONE);
