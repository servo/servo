// This file was procedurally generated from the following sources:
// - src/function-forms/params-trailing-comma-single.case
// - src/function-forms/default/cls-expr-async-meth-static.template
/*---
description: A trailing comma should not increase the respective length, using a single parameter (static class expression async method)
esid: sec-class-definitions-runtime-semantics-evaluation
features: [async-functions]
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
      b. Else,
        Let status be the result of performing PropertyDefinitionEvaluation
        for m with arguments F and false.
    [...]

    Runtime Semantics: PropertyDefinitionEvaluation

    AsyncMethod : async PropertyName ( UniqueFormalParameters ) { AsyncFunctionBody }

    1. Let propKey be the result of evaluating PropertyName.
    2. ReturnIfAbrupt(propKey).
    3. If the function code for this AsyncMethod is strict mode code, let strict be true. Otherwise
       let strict be false.
    4. Let scope be the LexicalEnvironment of the running execution context.
    5. Let closure be ! AsyncFunctionCreate(Method, UniqueFormalParameters, AsyncFunctionBody,
       scope, strict).
    [...]


    Trailing comma in the parameters list

    14.1 Function Definitions

    FormalParameters[Yield, Await] : FormalParameterList[?Yield, ?Await] ,
---*/

var callCount = 0;
var C = class {
  static async method(a,) {
    assert.sameValue(a, 42);
    callCount = callCount + 1;
  }
}

// Stores a reference `ref` for case evaluation
var ref = C.method;

ref(42, 39).then(() => {
  assert.sameValue(callCount, 1, 'method invoked exactly once');
}).then($DONE, $DONE);

assert.sameValue(ref.length, 1, 'length is properly set');
