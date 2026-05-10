// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-elem-id-iter-complete.case
// - src/dstr-binding/default/arrow-function.template
/*---
description: SingleNameBinding when value iteration completes (arrow function expression)
esid: sec-arrow-function-definitions-runtime-semantics-evaluation
features: [destructuring-binding]
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

    13.3.3.6 Runtime Semantics: IteratorBindingInitialization

    SingleNameBinding : BindingIdentifier Initializeropt

    [...]
    4. If iteratorRecord.[[done]] is false, then
       a. Let next be IteratorStep(iteratorRecord.[[iterator]]).
       b. If next is an abrupt completion, set iteratorRecord.[[done]] to true.
       c. ReturnIfAbrupt(next).
       d. If next is false, set iteratorRecord.[[done]] to true.
       e. Else,
          [...]
    5. If iteratorRecord.[[done]] is true, let v be undefined.
    [...]
    8. Return InitializeReferencedBinding(lhs, v).
---*/

var callCount = 0;
var f;
f = ([x]) => {
  assert.sameValue(x, undefined);
  callCount = callCount + 1;
};

f([]);
assert.sameValue(callCount, 1, 'arrow function invoked exactly once');
