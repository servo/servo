// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-prop-ary.case
// - src/dstr-binding/default/gen-func-expr-dflt.template
/*---
description: Object binding pattern with "nested" array binding pattern not using initializer (generator function expression (default parameter))
esid: sec-generator-function-definitions-runtime-semantics-evaluation
features: [generators, destructuring-binding, default-parameters]
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

    13.3.3.7 Runtime Semantics: KeyedBindingInitialization

    [...]
    3. If Initializer is present and v is undefined, then
       [...]
    4. Return the result of performing BindingInitialization for BindingPattern
       passing v and environment as arguments.
---*/

var callCount = 0;
var f;
f = function*({ w: [x, y, z] = [4, 5, 6] } = { w: [7, undefined, ] }) {
  assert.sameValue(x, 7);
  assert.sameValue(y, undefined);
  assert.sameValue(z, undefined);

  assert.throws(ReferenceError, function() {
    w;
  });
  callCount = callCount + 1;
};

f().next();
assert.sameValue(callCount, 1, 'generator function invoked exactly once');
