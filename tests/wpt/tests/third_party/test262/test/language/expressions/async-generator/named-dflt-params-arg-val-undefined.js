// This file was procedurally generated from the following sources:
// - src/function-forms/dflt-params-arg-val-undefined.case
// - src/function-forms/default/async-gen-named-func-expr.template
/*---
description: Use of initializer when argument value is `undefined` (async generator named function expression)
esid: sec-asyncgenerator-definitions-evaluation
features: [default-parameters, async-iteration]
flags: [generated, async]
info: |
    AsyncGeneratorExpression : async [no LineTerminator here] function * BindingIdentifier
        ( FormalParameters ) { AsyncGeneratorBody }

        [...]
        7. Let closure be ! AsyncGeneratorFunctionCreate(Normal, FormalParameters,
           AsyncGeneratorBody, funcEnv, strict).
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
// Stores a reference `ref` for case evaluation
var ref;
ref = async function* g(fromLiteral = 23, fromExpr = 45, fromHole = 99) {
  assert.sameValue(fromLiteral, 23);
  assert.sameValue(fromExpr, 45);
  assert.sameValue(fromHole, 99);
  callCount = callCount + 1;
};

ref(undefined, void 0).next().then(() => {
    assert.sameValue(callCount, 1, 'generator function invoked exactly once');
}).then($DONE, $DONE);
