// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-rest-val-obj.case
// - src/dstr-binding/default/arrow-function-dflt.template
/*---
description: Rest object contains just unextracted data (arrow function expression (default parameter))
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

var callCount = 0;
var f;
f = ({a, b, ...rest} = {x: 1, y: 2, a: 5, b: 3}) => {
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

f();
assert.sameValue(callCount, 1, 'arrow function invoked exactly once');
