// This file was procedurally generated from the following sources:
// - src/function-forms/dflt-params-ref-later.case
// - src/function-forms/error/async-gen-func-decl.template
/*---
description: Referencing a parameter that occurs later in the ParameterList (async generator function declaration)
esid: sec-asyncgenerator-definitions-instantiatefunctionobject
features: [default-parameters, async-iteration]
flags: [generated]
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
async function* f(x = y, y) {
  
  callCount = callCount + 1;
}

assert.throws(ReferenceError, function() {
  f();
});

assert.sameValue(callCount, 0, 'generator function body not evaluated');
