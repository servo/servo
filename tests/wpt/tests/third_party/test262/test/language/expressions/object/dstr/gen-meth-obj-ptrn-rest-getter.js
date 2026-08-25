// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-rest-getter.case
// - src/dstr-binding/default/gen-meth.template
/*---
description: Getter is called when obj is being deconstructed to a rest Object (generator method)
esid: sec-generator-function-definitions-runtime-semantics-propertydefinitionevaluation
features: [object-rest, generators, destructuring-binding]
flags: [generated]
includes: [propertyHelper.js]
info: |
    GeneratorMethod :
        * PropertyName ( StrictFormalParameters ) { GeneratorBody }

    1. Let propKey be the result of evaluating PropertyName.
    2. ReturnIfAbrupt(propKey).
    3. If the function code for this GeneratorMethod is strict mode code,
       let strict be true. Otherwise let strict be false.
    4. Let scope be the running execution context's LexicalEnvironment.
    5. Let closure be GeneratorFunctionCreate(Method,
       StrictFormalParameters, GeneratorBody, scope, strict).
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
var obj = {
  *method({...x}) {
    assert.sameValue(count, 1);

    verifyProperty(x, "v", {
      enumerable: true,
      writable: true,
      configurable: true,
      value: 2
    });
    callCount = callCount + 1;
  }
};

obj.method({ get v() { count++; return 2; } }).next();
assert.sameValue(callCount, 1, 'generator method invoked exactly once');
