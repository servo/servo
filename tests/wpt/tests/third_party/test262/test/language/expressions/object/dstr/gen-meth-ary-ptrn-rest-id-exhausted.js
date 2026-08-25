// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-rest-id-exhausted.case
// - src/dstr-binding/default/gen-meth.template
/*---
description: RestElement applied to an exhausted iterator (generator method)
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

    13.3.3.6 Runtime Semantics: IteratorBindingInitialization
    BindingRestElement : ... BindingIdentifier
    1. Let lhs be ResolveBinding(StringValue of BindingIdentifier,
       environment).
    2. ReturnIfAbrupt(lhs). 3. Let A be ArrayCreate(0). 4. Let n=0. 5. Repeat,
       [...]
       b. If iteratorRecord.[[done]] is true, then
          i. If environment is undefined, return PutValue(lhs, A).
          ii. Return InitializeReferencedBinding(lhs, A).

---*/

var callCount = 0;
var obj = {
  *method([, , ...x]) {
    assert(Array.isArray(x));
    assert.sameValue(x.length, 0);
    callCount = callCount + 1;
  }
};

obj.method([1, 2]).next();
assert.sameValue(callCount, 1, 'generator method invoked exactly once');
