// This file was procedurally generated from the following sources:
// - src/function-forms/dflt-params-trailing-comma.case
// - src/function-forms/default/cls-expr-async-gen-meth.template
/*---
description: A trailing comma should not increase the respective length, using default parameters (class expression async generator method)
esid: sec-class-definitions-runtime-semantics-evaluation
features: [async-iteration]
flags: [generated, async]
info: |
    ClassExpression : class BindingIdentifieropt ClassTail

    1. If BindingIdentifieropt is not present, let className be undefined.
    2. Else, let className be StringValue of BindingIdentifier.
    3. Let value be the result of ClassDefinitionEvaluation of ClassTail
       with argument className.
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


    Trailing comma in the parameters list

    14.1 Function Definitions

    FormalParameters[Yield, Await] : FormalParameterList[?Yield, ?Await] ,
---*/

var callCount = 0;
var C = class {
  async *method(a, b = 39,) {
    assert.sameValue(a, 42);
    assert.sameValue(b, 39);
    callCount = callCount + 1;
  }
}

// Stores a reference `ref` for case evaluation
var ref = C.prototype.method;

ref(42, undefined, 1).next().then(() => {
    assert.sameValue(callCount, 1, 'method invoked exactly once');
}).then($DONE, $DONE);


assert.sameValue(ref.length, 1, 'length is properly set');
