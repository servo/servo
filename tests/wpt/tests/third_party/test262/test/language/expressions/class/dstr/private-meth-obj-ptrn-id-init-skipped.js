// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-id-init-skipped.case
// - src/dstr-binding/default/cls-expr-private-meth.template
/*---
description: Destructuring initializer is not evaluated when value is not `undefined` (private class expression method)
esid: sec-class-definitions-runtime-semantics-evaluation
features: [class, class-methods-private, destructuring-binding]
flags: [generated]
info: |
    ClassExpression : class BindingIdentifieropt ClassTail

    1. If BindingIdentifieropt is not present, let className be undefined.
    2. Else, let className be StringValue of BindingIdentifier.
    3. Let value be the result of ClassDefinitionEvaluation of ClassTail
       with argument className.
    [...]

    14.5.14 Runtime Semantics: ClassDefinitionEvaluation

    21. For each ClassElement m in order from methods
        a. If IsStatic of m is false, then
           i. Let status be the result of performing
              PropertyDefinitionEvaluation for m with arguments proto and
              false.
        [...]

    14.3.8 Runtime Semantics: DefineMethod

    MethodDefinition : PropertyName ( StrictFormalParameters ) { FunctionBody }

    [...]
    6. Let closure be FunctionCreate(kind, StrictFormalParameters, FunctionBody,
       scope, strict). If functionPrototype was passed as a parameter then pass its
       value as the functionPrototype optional argument of FunctionCreate.
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

    SingleNameBinding : BindingIdentifier Initializeropt

    [...]
    6. If Initializer is present and v is undefined, then
       [...]
    [...]
---*/
var initCount = 0;
function counter() {
  initCount += 1;
}

var callCount = 0;
var C = class {
  #method({ w = counter(), x = counter(), y = counter(), z = counter() }) {
    assert.sameValue(w, null);
    assert.sameValue(x, 0);
    assert.sameValue(y, false);
    assert.sameValue(z, '');
    assert.sameValue(initCount, 0);
    callCount = callCount + 1;
  }

  get method() {
    return this.#method;
  }
};

new C().method({ w: null, x: 0, y: false, z: '' });
assert.sameValue(callCount, 1, 'method invoked exactly once');
