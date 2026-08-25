// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-rest-id-direct.case
// - src/dstr-binding/default/func-decl-dflt.template
/*---
description: Lone rest element (direct binding) (function declaration (default parameter))
esid: sec-function-definitions-runtime-semantics-instantiatefunctionobject
features: [destructuring-binding, default-parameters]
flags: [generated]
includes: [compareArray.js]
info: |
    FunctionDeclaration :
        function BindingIdentifier ( FormalParameters ) { FunctionBody }

        [...]
        3. Let F be FunctionCreate(Normal, FormalParameters, FunctionBody,
           scope, strict).
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

    Runtime Semantics: IteratorBindingInitialization

    BindingRestElement : ... BindingIdentifier

    [...]
    2. Let A be ! ArrayCreate(0).
    3. Let n be 0.
    4. Repeat,
        [...]
        f. Perform ! CreateDataPropertyOrThrow(A, ! ToString(n), nextValue).
        g. Set n to n + 1.

---*/

var callCount = 0;
function f([...x] = [1]) {
  assert(Array.isArray(x));
  assert.compareArray(x, [1]);
  callCount = callCount + 1;
};
f();
assert.sameValue(callCount, 1, 'function invoked exactly once');
