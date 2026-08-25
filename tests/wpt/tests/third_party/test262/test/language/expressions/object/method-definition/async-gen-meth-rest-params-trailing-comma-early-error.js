// This file was procedurally generated from the following sources:
// - src/function-forms/rest-params-trailing-comma-early-error.case
// - src/function-forms/syntax/async-gen-meth.template
/*---
description: It's a syntax error if a FunctionRestParameter is followed by a trailing comma (async generator method)
esid: sec-asyncgenerator-definitions-propertydefinitionevaluation
features: [async-iteration]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
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


    Trailing comma in the parameters list

    14.1 Function Definitions

    FormalParameters[Yield, Await] :
        [empty]
        FunctionRestParameter[?Yield, ?Await]
        FormalParameterList[?Yield, ?Await]
        FormalParameterList[?Yield, ?Await] ,
        FormalParameterList[?Yield, ?Await] , FunctionRestParameter[?Yield, ?Await]
---*/
$DONOTEVALUATE();

0, {
  async *method(...a,) {
    
  }
};
