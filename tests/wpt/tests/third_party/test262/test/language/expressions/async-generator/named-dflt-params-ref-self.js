// This file was procedurally generated from the following sources:
// - src/function-forms/dflt-params-ref-self.case
// - src/function-forms/error/async-gen-named-func-expr.template
/*---
description: Referencing a parameter from within its own initializer (async generator named function expression)
esid: sec-asyncgenerator-definitions-evaluation
features: [default-parameters, async-iteration]
flags: [generated]
info: |
    AsyncGeneratorExpression : async [no LineTerminator here] function * BindingIdentifier
        ( FormalParameters ) { AsyncGeneratorBody }

        [...]
        7. Let closure be ! AsyncGeneratorFunctionCreate(Normal, FormalParameters,
           AsyncGeneratorBody, funcEnv, strict).
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
var f;
f = async function* g(x = x) {
  
  callCount = callCount + 1;
};

assert.throws(ReferenceError, function() {
  f();
});
assert.sameValue(callCount, 0, 'generator function body not evaluated');
