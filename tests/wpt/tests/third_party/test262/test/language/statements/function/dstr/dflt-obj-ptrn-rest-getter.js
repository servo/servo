// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-rest-getter.case
// - src/dstr-binding/default/func-decl-dflt.template
/*---
description: Getter is called when obj is being deconstructed to a rest Object (function declaration (default parameter))
esid: sec-function-definitions-runtime-semantics-instantiatefunctionobject
features: [object-rest, destructuring-binding, default-parameters]
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
var count = 0;

var callCount = 0;
function f({...x} = { get v() { count++; return 2; } }) {
  assert.sameValue(count, 1);

  verifyProperty(x, "v", {
    enumerable: true,
    writable: true,
    configurable: true,
    value: 2
  });
  callCount = callCount + 1;
};
f();
assert.sameValue(callCount, 1, 'function invoked exactly once');
