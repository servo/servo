// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-empty.case
// - src/dstr-binding/default/async-gen-func-decl-dflt.template
/*---
description: No iteration occurs for an "empty" array binding pattern (async generator function declaration (default parameter))
esid: sec-asyncgenerator-definitions-instantiatefunctionobject
features: [generators, async-iteration]
flags: [generated, async]
info: |
    AsyncGeneratorDeclaration : async [no LineTerminator here] function * BindingIdentifier
        ( FormalParameters ) { AsyncGeneratorBody }

        [...]
        3. Let F be ! AsyncGeneratorFunctionCreate(Normal, FormalParameters, AsyncGeneratorBody,
            scope, strict).
        [...]


    13.3.3.6 Runtime Semantics: IteratorBindingInitialization

    ArrayBindingPattern : [ ]

    1. Return NormalCompletion(empty).

---*/
var iterations = 0;
var iter = function*() {
  iterations += 1;
}();


var callCount = 0;
async function* f([] = iter) {
  assert.sameValue(iterations, 0);
  callCount = callCount + 1;
};
f().next().then(() => {
    assert.sameValue(callCount, 1, 'invoked exactly once');
}).then($DONE, $DONE);
