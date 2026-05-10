// This file was procedurally generated from the following sources:
// - src/function-forms/params-trailing-comma-single.case
// - src/function-forms/default/gen-func-expr.template
/*---
description: A trailing comma should not increase the respective length, using a single parameter (generator function expression)
esid: sec-generator-function-definitions-runtime-semantics-evaluation
features: [generators]
flags: [generated]
info: |
    GeneratorExpression : function * ( FormalParameters ) { GeneratorBody }

        [...]
        3. Let closure be GeneratorFunctionCreate(Normal, FormalParameters,
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


    Trailing comma in the parameters list

    14.1 Function Definitions

    FormalParameters[Yield, Await] : FormalParameterList[?Yield, ?Await] ,
---*/

var callCount = 0;
// Stores a reference `ref` for case evaluation
var ref;
ref = function*(a,) {
  assert.sameValue(a, 42);
  callCount = callCount + 1;
};

ref(42, 39).next();

assert.sameValue(callCount, 1, 'generator function invoked exactly once');

assert.sameValue(ref.length, 1, 'length is properly set');
