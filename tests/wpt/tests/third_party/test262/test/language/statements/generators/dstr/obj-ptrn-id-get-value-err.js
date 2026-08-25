// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-id-get-value-err.case
// - src/dstr-binding/error/gen-func-decl.template
/*---
description: Error thrown when accessing the corresponding property of the value object (generator function declaration)
esid: sec-generator-function-definitions-runtime-semantics-instantiatefunctionobject
features: [generators, destructuring-binding]
flags: [generated]
info: |
    GeneratorDeclaration : function * ( FormalParameters ) { GeneratorBody }

        [...]
        2. Let F be GeneratorFunctionCreate(Normal, FormalParameters,
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

    SingleNameBinding : BindingIdentifier Initializeropt

    [...]
    4. Let v be GetV(value, propertyName).
    5. ReturnIfAbrupt(v).
---*/
var poisonedProperty = Object.defineProperty({}, 'poisoned', {
  get: function() {
    throw new Test262Error();
  }
});

function* f({ poisoned }) {}

assert.throws(Test262Error, function() {
  f(poisonedProperty);
});
