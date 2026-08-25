// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-empty.case
// - src/dstr-binding/default/async-gen-func-expr-dflt.template
/*---
description: No property access occurs for an "empty" object binding pattern (async generator function expression (default parameter))
esid: sec-asyncgenerator-definitions-evaluation
features: [async-iteration]
flags: [generated, async]
info: |
    AsyncGeneratorExpression : async [no LineTerminator here] function * ( FormalParameters ) {
        AsyncGeneratorBody }

        [...]
        3. Let closure be ! AsyncGeneratorFunctionCreate(Normal, FormalParameters,
           AsyncGeneratorBody, scope, strict).
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
var f;
f = async function*({} = obj) {
  assert.sameValue(accessCount, 0);
  callCount = callCount + 1;
};

f().next().then(() => {
    assert.sameValue(callCount, 1, 'invoked exactly once');
}).then($DONE, $DONE);
