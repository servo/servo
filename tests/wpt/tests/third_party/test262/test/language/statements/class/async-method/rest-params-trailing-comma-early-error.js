// This file was procedurally generated from the following sources:
// - src/function-forms/rest-params-trailing-comma-early-error.case
// - src/function-forms/syntax/cls-decl-async-meth.template
/*---
description: It's a syntax error if a FunctionRestParameter is followed by a trailing comma (class declaration async method)
esid: sec-class-definitions-runtime-semantics-evaluation
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
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

    FormalParameters[Yield, Await] :
        [empty]
        FunctionRestParameter[?Yield, ?Await]
        FormalParameterList[?Yield, ?Await]
        FormalParameterList[?Yield, ?Await] ,
        FormalParameterList[?Yield, ?Await] , FunctionRestParameter[?Yield, ?Await]
---*/
$DONOTEVALUATE();


class C {
  async method(...a,) {
    
  }
}
