// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-init-undefined.case
// - src/dstr-binding/error/cls-decl-async-gen-meth-dflt.template
/*---
description: Value specifed for object binding pattern must be object coercible (undefined) (class expression async generator method (default parameters))
esid: sec-class-definitions-runtime-semantics-evaluation
features: [async-iteration]
flags: [generated]
info: |
    ClassDeclaration : class BindingIdentifier ClassTail

    1. Let className be StringValue of BindingIdentifier.
    2. Let value be the result of ClassDefinitionEvaluation of ClassTail with
       argument className.
    [...]

    14.5.14 Runtime Semantics: ClassDefinitionEvaluation

    21. For each ClassElement m in order from methods
        a. If IsStatic of m is false, then
           i. Let status be the result of performing
              PropertyDefinitionEvaluation for m with arguments proto and
              false.
        [...]

    Runtime Semantics: PropertyDefinitionEvaluation

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

    Runtime Semantics: BindingInitialization

    ObjectBindingPattern : { }

    1. Return NormalCompletion(empty).
---*/


class C {
  async *method({} = undefined) {
    
  }
};

var method = C.prototype.method;

assert.throws(TypeError, function() {
  method();
});
