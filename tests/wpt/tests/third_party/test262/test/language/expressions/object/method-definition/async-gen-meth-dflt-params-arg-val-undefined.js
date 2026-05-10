// This file was procedurally generated from the following sources:
// - src/function-forms/dflt-params-arg-val-undefined.case
// - src/function-forms/default/async-gen-meth.template
/*---
description: Use of initializer when argument value is `undefined` (async generator method)
esid: sec-asyncgenerator-definitions-propertydefinitionevaluation
features: [default-parameters, async-iteration]
flags: [generated, async]
info: |
    AsyncGeneratorMethod :
        async [no LineTerminator here] * PropertyName ( UniqueFormalParameters )
            { AsyncGeneratorBody }

    1. Let propKey be the result of evaluating PropertyName.
    2. ReturnIfAbrupt(propKey).
    3. If the function code for this AsyncGeneratorMethod is strict mode code, let strict be true.
       Otherwise let strict be false.
    4. Let scope be the running execution context's LexicalEnvironment.
    5. Let closure be ! AsyncGeneratorFunctionCreate(Method, UniqueFormalParameters,
       AsyncGeneratorBody, scope, strict).
    [...]


    14.1.19 Runtime Semantics: IteratorBindingInitialization

    FormalsList : FormalsList , FormalParameter

    [...]
    23. Let iteratorRecord be Record {[[Iterator]]:
        CreateListIterator(argumentsList), [[Done]]: false}.
    24. If hasDuplicates is true, then
        [...]
    25. Else,
        a. Perform ? IteratorBindingInitialization for formals with
           iteratorRecord and env as arguments.
    [...]

---*/

var callCount = 0;
var obj = {
  async *method(fromLiteral = 23, fromExpr = 45, fromHole = 99) {
    assert.sameValue(fromLiteral, 23);
    assert.sameValue(fromExpr, 45);
    assert.sameValue(fromHole, 99);
    callCount = callCount + 1;
  }
};

// Stores a reference `ref` for case evaluation
var ref = obj.method;

ref(undefined, void 0).next().then(() => {
    assert.sameValue(callCount, 1, 'generator method invoked exactly once');
}).then($DONE, $DONE);
