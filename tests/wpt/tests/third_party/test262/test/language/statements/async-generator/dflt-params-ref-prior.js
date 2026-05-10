// This file was procedurally generated from the following sources:
// - src/function-forms/dflt-params-ref-prior.case
// - src/function-forms/default/async-gen-func-decl.template
/*---
description: Referencing a parameter that occurs earlier in the ParameterList (async generator function declaration)
esid: sec-asyncgenerator-definitions-instantiatefunctionobject
features: [default-parameters, async-iteration]
flags: [generated, async]
info: |
    AsyncGeneratorDeclaration : async [no LineTerminator here] function * BindingIdentifier
        ( FormalParameters ) { AsyncGeneratorBody }

        [...]
        3. Let F be ! AsyncGeneratorFunctionCreate(Normal, FormalParameters, AsyncGeneratorBody,
            scope, strict).
        [...]


    14.1.19 Runtime Semantics: IteratorBindingInitialization

    FormalsList : FormalsList , FormalParameter

    1. Let status be the result of performing IteratorBindingInitialization for
       FormalsList using iteratorRecord and environment as the arguments.
    2. ReturnIfAbrupt(status).
    3. Return the result of performing IteratorBindingInitialization for
       FormalParameter using iteratorRecord and environment as the arguments.

---*/
var x = 0;


var callCount = 0;
// Stores a reference `ref` for case evaluation
async function* ref(x, y = x, z = y) {
  assert.sameValue(x, 3, 'first argument value');
  assert.sameValue(y, 3, 'second argument value');
  assert.sameValue(z, 3, 'third argument value');
  callCount = callCount + 1;
}

ref(3).next().then(() => {
    assert.sameValue(callCount, 1, 'generator function invoked exactly once');
}).then($DONE, $DONE);
