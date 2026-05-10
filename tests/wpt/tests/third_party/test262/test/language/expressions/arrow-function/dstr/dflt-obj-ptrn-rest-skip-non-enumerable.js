// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-rest-skip-non-enumerable.case
// - src/dstr-binding/default/arrow-function-dflt.template
/*---
description: Rest object doesn't contain non-enumerable properties (arrow function expression (default parameter))
esid: sec-arrow-function-definitions-runtime-semantics-evaluation
features: [object-rest, destructuring-binding, default-parameters]
flags: [generated]
includes: [propertyHelper.js]
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
---*/
var o = {a: 3, b: 4};
Object.defineProperty(o, "x", { value: 4, enumerable: false });

var callCount = 0;
var f;
f = ({...rest} = o) => {
  assert.sameValue(rest.x, undefined);

  verifyProperty(rest, "a", {
    enumerable: true,
    writable: true,
    configurable: true,
    value: 3
  });

  verifyProperty(rest, "b", {
    enumerable: true,
    writable: true,
    configurable: true,
    value: 4
  });
  callCount = callCount + 1;
};

f();
assert.sameValue(callCount, 1, 'arrow function invoked exactly once');
