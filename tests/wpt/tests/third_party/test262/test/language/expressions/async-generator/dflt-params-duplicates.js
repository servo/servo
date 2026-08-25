// This file was procedurally generated from the following sources:
// - src/function-forms/dflt-params-duplicates.case
// - src/function-forms/syntax/async-gen-func-expr.template
/*---
description: It is a Syntax Error if BoundNames of FormalParameters contains any duplicate elements. (async generator function expression)
esid: sec-asyncgenerator-definitions-evaluation
features: [default-parameters, async-iteration]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    AsyncGeneratorExpression : async [no LineTerminator here] function * ( FormalParameters ) {
        AsyncGeneratorBody }

        [...]
        3. Let closure be ! AsyncGeneratorFunctionCreate(Normal, FormalParameters,
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


0, async function*(x = 0, x) {
  
};
