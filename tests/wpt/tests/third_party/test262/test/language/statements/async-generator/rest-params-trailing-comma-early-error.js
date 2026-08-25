// This file was procedurally generated from the following sources:
// - src/function-forms/rest-params-trailing-comma-early-error.case
// - src/function-forms/syntax/async-gen-func-decl.template
/*---
description: It's a syntax error if a FunctionRestParameter is followed by a trailing comma (async generator function declaration)
esid: sec-asyncgenerator-definitions-instantiatefunctionobject
features: [async-iteration]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    AsyncGeneratorDeclaration : async [no LineTerminator here] function * BindingIdentifier
        ( FormalParameters ) { AsyncGeneratorBody }

        [...]
        3. Let F be ! AsyncGeneratorFunctionCreate(Normal, FormalParameters, AsyncGeneratorBody,
            scope, strict).
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


async function* f(...a,) {
  
}
