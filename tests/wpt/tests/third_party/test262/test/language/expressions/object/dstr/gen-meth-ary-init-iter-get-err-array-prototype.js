// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-init-iter-get-err-array-prototype.case
// - src/dstr-binding/error/gen-meth.template
/*---
description: Abrupt completion returned by GetIterator (generator method)
esid: sec-generator-function-definitions-runtime-semantics-propertydefinitionevaluation
features: [Symbol.iterator, generators, destructuring-binding]
flags: [generated]
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

    Runtime Semantics: BindingInitialization

    BindingPattern : ArrayBindingPattern

    1. Let iteratorRecord be ? GetIterator(value).

    GetIterator ( obj [ , hint [ , method ] ] )

    [...]
    4. Let iterator be ? Call(method, obj).

    Call ( F, V [ , argumentsList ] )

    [...]
    2. If IsCallable(F) is false, throw a TypeError exception.

---*/
delete Array.prototype[Symbol.iterator];

var obj = {
  *method([x, y, z]) {}
};

assert.throws(TypeError, function() {
  obj.method([1, 2, 3]);
});
