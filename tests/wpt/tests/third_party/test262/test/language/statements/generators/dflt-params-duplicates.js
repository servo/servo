// This file was procedurally generated from the following sources:
// - src/function-forms/dflt-params-duplicates.case
// - src/function-forms/syntax/gen-func-decl.template
/*---
description: It is a Syntax Error if BoundNames of FormalParameters contains any duplicate elements. (generator function declaration)
esid: sec-generator-function-definitions-runtime-semantics-instantiatefunctionobject
features: [default-parameters, generators]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    GeneratorDeclaration : function * ( FormalParameters ) { GeneratorBody }

        [...]
        2. Let F be GeneratorFunctionCreate(Normal, FormalParameters,
           GeneratorBody, scope, strict).
        [...]

    9.2.1 [[Call]] ( thisArgument, argumentsList)

    [...]
    7. Let result be OrdinaryCallEvaluateBody(F, argumentsList).
    [...]

    9.2.1.3 OrdinaryCallEvaluateBody ( F, argumentsList )

    1. Let status be FunctionDeclarationInstantiation(F, argumentsList).
    [...]

    9.2.12 FunctionDeclarationInstantiation(func, argumentsList)

    [...]
    23. Let iteratorRecord be Record {[[iterator]]:
        CreateListIterator(argumentsList), [[done]]: false}.
    24. If hasDuplicates is true, then
        [...]
    25. Else,
        b. Let formalStatus be IteratorBindingInitialization for formals with
           iteratorRecord and env as arguments.
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

function* f(x = 0, x) {
  
}
