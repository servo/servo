// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-name-iter-val.case
// - src/dstr-binding/default/gen-meth.template
/*---
description: SingleNameBinding with normal value iteration (generator method)
esid: sec-generator-function-definitions-runtime-semantics-propertydefinitionevaluation
features: [generators, destructuring-binding]
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

    SingleNameBinding : BindingIdentifier Initializeropt

    [...]
    4. If iteratorRecord.[[done]] is false, then
       a. Let next be IteratorStep(iteratorRecord.[[iterator]]).
       b. If next is an abrupt completion, set iteratorRecord.[[done]] to true.
       c. ReturnIfAbrupt(next).
       d. If next is false, set iteratorRecord.[[done]] to true.
       e. Else,
          [...]
          i. Let v be IteratorValue(next).
          ii. If v is an abrupt completion, set
              iteratorRecord.[[done]] to true.
          iii. ReturnIfAbrupt(v).
    5. If iteratorRecord.[[done]] is true, let v be undefined.
    [...]
    8. Return InitializeReferencedBinding(lhs, v).
---*/

var callCount = 0;
var obj = {
  *method([x, y, z]) {
    assert.sameValue(x, 1);
    assert.sameValue(y, 2);
    assert.sameValue(z, 3);
    callCount = callCount + 1;
  }
};

obj.method([1, 2, 3]).next();
assert.sameValue(callCount, 1, 'generator method invoked exactly once');
