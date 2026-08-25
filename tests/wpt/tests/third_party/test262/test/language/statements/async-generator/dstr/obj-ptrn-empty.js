// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-empty.case
// - src/dstr-binding/default/async-gen-func-decl.template
/*---
description: No property access occurs for an "empty" object binding pattern (async generator function declaration)
esid: sec-asyncgenerator-definitions-instantiatefunctionobject
features: [async-iteration]
flags: [generated, async]
info: |
    AsyncGeneratorDeclaration : async [no LineTerminator here] function * BindingIdentifier
        ( FormalParameters ) { AsyncGeneratorBody }

        [...]
        3. Let F be ! AsyncGeneratorFunctionCreate(Normal, FormalParameters, AsyncGeneratorBody,
            scope, strict).
        [...]


    Runtime Semantics: BindingInitialization

    ObjectBindingPattern : { }

    1. Return NormalCompletion(empty).
---*/
var accessCount = 0;
var obj = Object.defineProperty({}, 'attr', {
  get: function() {
    accessCount += 1;
  }
});


var callCount = 0;
async function* f({}) {
  assert.sameValue(accessCount, 0);
  callCount = callCount + 1;
};
f(obj).next().then(() => {
    assert.sameValue(callCount, 1, 'invoked exactly once');
}).then($DONE, $DONE);
