// This file was procedurally generated from the following sources:
// - src/function-forms/dflt-params-ref-later.case
// - src/function-forms/error/gen-func-decl.template
/*---
description: Referencing a parameter that occurs later in the ParameterList (generator function declaration)
esid: sec-generator-function-definitions-runtime-semantics-instantiatefunctionobject
features: [default-parameters, generators]
flags: [generated]
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


    14.1.19 Runtime Semantics: IteratorBindingInitialization

    FormalsList : FormalsList , FormalParameter

    1. Let status be the result of performing IteratorBindingInitialization for
       FormalsList using iteratorRecord and environment as the arguments.
    2. ReturnIfAbrupt(status).
    3. Return the result of performing IteratorBindingInitialization for
       FormalParameter using iteratorRecord and environment as the arguments.

---*/
var x = 0;

var callCount = 0;
function* f(x = y, y) {
  
  callCount = callCount + 1;
}

assert.throws(ReferenceError, function() {
  f();
});

assert.sameValue(callCount, 0, 'generator function body not evaluated');
