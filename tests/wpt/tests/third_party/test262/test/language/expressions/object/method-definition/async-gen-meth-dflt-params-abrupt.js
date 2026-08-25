// This file was procedurally generated from the following sources:
// - src/function-forms/dflt-params-abrupt.case
// - src/function-forms/error/async-gen-meth.template
/*---
description: Abrupt completion returned by evaluation of initializer (async generator method)
esid: sec-asyncgenerator-definitions-propertydefinitionevaluation
features: [default-parameters, async-iteration]
flags: [generated]
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


    14.1.19 Runtime Semantics: IteratorBindingInitialization

    FormalsList : FormalsList , FormalParameter

    1. Let status be the result of performing IteratorBindingInitialization for
       FormalsList using iteratorRecord and environment as the arguments.
    2. ReturnIfAbrupt(status).
    3. Return the result of performing IteratorBindingInitialization for
       FormalParameter using iteratorRecord and environment as the arguments.

---*/

var callCount = 0;
var obj = {
  async *method(_ = (function() { throw new Test262Error(); }())) {
    
    callCount = callCount + 1;
  }
};

assert.throws(Test262Error, function() {
  obj.method();
});
assert.sameValue(callCount, 0, 'generator method body not evaluated');
