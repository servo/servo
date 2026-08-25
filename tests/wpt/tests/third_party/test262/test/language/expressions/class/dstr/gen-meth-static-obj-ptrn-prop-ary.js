// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-prop-ary.case
// - src/dstr-binding/default/cls-expr-gen-meth-static.template
/*---
description: Object binding pattern with "nested" array binding pattern not using initializer (static class expression generator method)
esid: sec-class-definitions-runtime-semantics-evaluation
features: [generators, destructuring-binding]
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
           Let status be the result of performing PropertyDefinitionEvaluation
           for m with arguments F and false.
    [...]

    14.4.13 Runtime Semantics: PropertyDefinitionEvaluation

    GeneratorMethod :
        * PropertyName ( StrictFormalParameters ) { GeneratorBody }

    1. Let propKey be the result of evaluating PropertyName.
    2. ReturnIfAbrupt(propKey).
    3. If the function code for this GeneratorMethod is strict mode code,
       let strict be true. Otherwise let strict be false.
    4. Let scope be the running execution context's LexicalEnvironment.
    5. Let closure be GeneratorFunctionCreate(Method,
       StrictFormalParameters, GeneratorBody, scope, strict).

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
  static *method({ w: [x, y, z] = [4, 5, 6] }) {
    assert.sameValue(x, 7);
    assert.sameValue(y, undefined);
    assert.sameValue(z, undefined);

    assert.throws(ReferenceError, function() {
      w;
    });
    callCount = callCount + 1;
  }
};

C.method({ w: [7, undefined, ] }).next();
assert.sameValue(callCount, 1, 'method invoked exactly once');
