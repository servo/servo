// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-elem-id-init-skipped.case
// - src/dstr-binding/default/arrow-function-dflt.template
/*---
description: Destructuring initializer is not evaluated when value is not `undefined` (arrow function expression (default parameter))
esid: sec-arrow-function-definitions-runtime-semantics-evaluation
features: [destructuring-binding, default-parameters]
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
    6. If Initializer is present and v is undefined, then
       [...]
    7. If environment is undefined, return PutValue(lhs, v).
    8. Return InitializeReferencedBinding(lhs, v).
---*/
var initCount = 0;
function counter() {
  initCount += 1;
}

var callCount = 0;
var f;
f = ([w = counter(), x = counter(), y = counter(), z = counter()] = [null, 0, false, '']) => {
  assert.sameValue(w, null);
  assert.sameValue(x, 0);
  assert.sameValue(y, false);
  assert.sameValue(z, '');
  assert.sameValue(initCount, 0);
  callCount = callCount + 1;
};

f();
assert.sameValue(callCount, 1, 'arrow function invoked exactly once');
