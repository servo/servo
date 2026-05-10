// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-prop-id-init-skipped.case
// - src/dstr-binding/default/func-expr.template
/*---
description: Destructuring initializer is not evaluated when value is not `undefined` (function expression)
esid: sec-function-definitions-runtime-semantics-evaluation
features: [destructuring-binding]
flags: [generated]
info: |
    FunctionExpression : function ( FormalParameters ) { FunctionBody }

        [...]
        3. Let closure be FunctionCreate(Normal, FormalParameters, FunctionBody,
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

    13.3.3.7 Runtime Semantics: KeyedBindingInitialization

    BindingElement : BindingPattern Initializeropt

    [...]
    3. If Initializer is present and v is undefined, then
    [...]
---*/
var initCount = 0;
function counter() {
  initCount += 1;
}

var callCount = 0;
var f;
f = function({ s: t = counter(), u: v = counter(), w: x = counter(), y: z = counter() }) {
  assert.sameValue(t, null);
  assert.sameValue(v, 0);
  assert.sameValue(x, false);
  assert.sameValue(z, '');
  assert.sameValue(initCount, 0);

  assert.throws(ReferenceError, function() {
    s;
  });
  assert.throws(ReferenceError, function() {
    u;
  });
  assert.throws(ReferenceError, function() {
    w;
  });
  assert.throws(ReferenceError, function() {
    y;
  });
  callCount = callCount + 1;
};

f({ s: null, u: 0, w: false, y: '' });
assert.sameValue(callCount, 1, 'function invoked exactly once');
