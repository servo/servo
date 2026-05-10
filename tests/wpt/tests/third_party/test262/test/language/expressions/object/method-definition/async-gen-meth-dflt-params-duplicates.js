// This file was procedurally generated from the following sources:
// - src/function-forms/dflt-params-duplicates.case
// - src/function-forms/syntax/async-gen-meth.template
/*---
description: It is a Syntax Error if BoundNames of FormalParameters contains any duplicate elements. (async generator method)
esid: sec-asyncgenerator-definitions-propertydefinitionevaluation
features: [default-parameters, async-iteration]
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


    14.1.2 Static Semantics: Early Errors

    StrictFormalParameters : FormalParameters

    - It is a Syntax Error if BoundNames of FormalParameters contains any
      duplicate elements.

    FormalParameters : FormalParameterList

    - It is a Syntax Error if IsSimpleParameterList of FormalParameterList is
      false and BoundNames of FormalParameterList contains any duplicate
      elements.

---*/
$DONOTEVALUATE();

0, {
  async *method(x = 0, x) {
    
  }
};
