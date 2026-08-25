// This file was procedurally generated from the following sources:
// - src/function-forms/rest-params-trailing-comma-early-error.case
// - src/function-forms/syntax/async-meth.template
/*---
description: It's a syntax error if a FunctionRestParameter is followed by a trailing comma (async method)
esid: sec-async-function-definitions
features: [async-iteration]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    14.6 Async Function Definitions

    AsyncMethod :
     async PropertyName ( UniqueFormalParameters ) { AsyncFunctionBody }


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


({
  async *method(...a,) {
    
  }
});
