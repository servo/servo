// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-prop-obj.case
// - src/dstr-binding/default/cls-expr-private-meth-static-dflt.template
/*---
description: Object binding pattern with "nested" object binding pattern not using initializer (private static class expression method (default parameter))
esid: sec-class-definitions-runtime-semantics-evaluation
features: [class, class-static-methods-private, destructuring-binding, default-parameters]
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
        b. Else,
           Let status be the result of performing PropertyDefinitionEvaluation for
           m with arguments F and false.
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

    [...]
    3. If Initializer is present and v is undefined, then
       [...]
    4. Return the result of performing BindingInitialization for BindingPattern
       passing v and environment as arguments.
---*/

var callCount = 0;
var C = class {
  static #method({ w: { x, y, z } = { x: 4, y: 5, z: 6 } } = { w: { x: undefined, z: 7 } }) {
    assert.sameValue(x, undefined);
    assert.sameValue(y, undefined);
    assert.sameValue(z, 7);

    assert.throws(ReferenceError, function() {
      w;
    });
    callCount = callCount + 1;
  }

  static get method() {
    return this.#method;
  }
};

C.method();
assert.sameValue(callCount, 1, 'method invoked exactly once');
