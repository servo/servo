// This file was procedurally generated from the following sources:
// - src/function-forms/params-trailing-comma-multiple.case
// - src/function-forms/default/arrow-function.template
/*---
description: A trailing comma should not increase the respective length, using multiple parameters (arrow function expression)
esid: sec-arrow-function-definitions-runtime-semantics-evaluation
flags: [generated]
info: |
    ArrowFunction : ArrowParameters => ConciseBody

    [...]
    4. Let closure be FunctionCreate(Arrow, parameters, ConciseBody, scope, strict).
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
ref = (a, b,) => {
  assert.sameValue(a, 42);
  assert.sameValue(b, 39);
  callCount = callCount + 1;
};

ref(42, 39, 1);
assert.sameValue(callCount, 1, 'arrow function invoked exactly once');

assert.sameValue(ref.length, 2, 'length is properly set');
