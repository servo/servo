// This file was procedurally generated from the following sources:
// - src/function-forms/dflt-params-rest.case
// - src/function-forms/syntax/async-gen-func-expr.template
/*---
description: RestParameter does not support an initializer (async generator function expression)
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


    14.1 Function Definitions

    Syntax

    FunctionRestParameter[Yield] :

      BindingRestElement[?Yield]

    13.3.3 Destructuring Binding Patterns

    Syntax

    BindingRestElement[Yield] :

      ...BindingIdentifier[?Yield]
      ...BindingPattern[?Yield]

---*/
$DONOTEVALUATE();


0, async function*(...x = []) {
  
};
