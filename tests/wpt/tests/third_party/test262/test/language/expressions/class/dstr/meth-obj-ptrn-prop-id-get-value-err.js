// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-prop-id-get-value-err.case
// - src/dstr-binding/error/cls-expr-meth.template
/*---
description: Error thrown when accessing the corresponding property of the value object (class expression method)
esid: sec-class-definitions-runtime-semantics-evaluation
features: [destructuring-binding]
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

    BindingElement : BindingPattern Initializeropt

    1. Let v be GetV(value, propertyName).
    2. ReturnIfAbrupt(v).
---*/
var initEvalCount = 0;
var poisonedProperty = Object.defineProperty({}, 'poisoned', {
  get: function() {
    throw new Test262Error();
  }
});

var C = class {
  method({ poisoned: x = ++initEvalCount }) {}
};

var c = new C();

assert.throws(Test262Error, function() {
  c.method(poisonedProperty);
});

assert.sameValue(initEvalCount, 0);
