// This file was procedurally generated from the following sources:
// - src/function-forms/dflt-params-rest.case
// - src/function-forms/syntax/async-meth.template
/*---
description: RestParameter does not support an initializer (async method)
esid: sec-async-function-definitions
features: [default-parameters, async-iteration]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    14.6 Async Function Definitions

    AsyncMethod :
     async PropertyName ( UniqueFormalParameters ) { AsyncFunctionBody }


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


({
  async *method(...x = []) {
    
  }
});
