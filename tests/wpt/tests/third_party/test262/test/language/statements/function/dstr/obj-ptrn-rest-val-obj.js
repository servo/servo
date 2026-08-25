// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-rest-val-obj.case
// - src/dstr-binding/default/func-decl.template
/*---
description: Rest object contains just unextracted data (function declaration)
esid: sec-function-definitions-runtime-semantics-instantiatefunctionobject
features: [object-rest, destructuring-binding]
flags: [generated]
includes: [propertyHelper.js]
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
---*/

var callCount = 0;
function f({a, b, ...rest}) {
  assert.sameValue(rest.a, undefined);
  assert.sameValue(rest.b, undefined);

  verifyProperty(rest, "x", {
    enumerable: true,
    writable: true,
    configurable: true,
    value: 1
  });

  verifyProperty(rest, "y", {
    enumerable: true,
    writable: true,
    configurable: true,
    value: 2
  });
  callCount = callCount + 1;
};
f({x: 1, y: 2, a: 5, b: 3});
assert.sameValue(callCount, 1, 'function invoked exactly once');
