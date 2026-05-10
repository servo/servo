// This file was procedurally generated from the following sources:
// - src/function-forms/unscopables-with-in-nested-fn.case
// - src/function-forms/default/func-decl.template
/*---
description: Symbol.unscopables behavior across scope boundaries (function declaration)
esid: sec-function-definitions-runtime-semantics-instantiatefunctionobject
features: [globalThis, Symbol.unscopables]
flags: [generated, noStrict]
info: |
    FunctionDeclaration :
        function BindingIdentifier ( FormalParameters ) { FunctionBody }

        [...]
        3. Let F be FunctionCreate(Normal, FormalParameters, FunctionBody,
           scope, strict).
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

    ...
    Let envRec be lex's EnvironmentRecord.
    Let exists be ? envRec.HasBinding(name).

    HasBinding

    ...
    If the withEnvironment flag of envRec is false, return true.
    Let unscopables be ? Get(bindings, @@unscopables).
    If Type(unscopables) is Object, then
       Let blocked be ToBoolean(? Get(unscopables, N)).
       If blocked is true, return false.

    (The `with` Statement) Runtime Semantics: Evaluation

    ...
    Set the withEnvironment flag of newEnv’s EnvironmentRecord to true.
    ...

---*/
let count = 0;
var v = 1;
globalThis[Symbol.unscopables] = {
  v: true,
};

{
  count++;

var callCount = 0;
// Stores a reference `ref` for case evaluation
function ref(x) {
  (function() {
    count++;
    with (globalThis) {
      count++;
      assert.sameValue(v, 1, 'The value of `v` is 1');
    }
  })();
  (function() {
    count++;
    var v = x;
    with (globalThis) {
      count++;
      assert.sameValue(v, 10, 'The value of `v` is 10');
      v = 20;
    }
    assert.sameValue(v, 20, 'The value of `v` is 20');
    assert.sameValue(globalThis.v, 1, 'The value of globalThis.v is 1');
  })();
  assert.sameValue(v, 1, 'The value of `v` is 1');
  assert.sameValue(globalThis.v, 1, 'The value of globalThis.v is 1');
  callCount = callCount + 1;
}

ref(10);

assert.sameValue(callCount, 1, 'function invoked exactly once');

  count++;
}
assert.sameValue(count, 6, 'The value of `count` is 6');
