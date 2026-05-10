// This file was procedurally generated from the following sources:
// - src/dynamic-import/eval-gtbndng-indirect-update-dflt.case
// - src/dynamic-import/default/nested-if-braceless.template
/*---
description: Modifications to default binding that occur after dependency has been evaluated are reflected in local binding (nested if syntax)
esid: sec-import-call-runtime-semantics-evaluation
features: [dynamic-import]
flags: [generated, async]
info: |
    ImportCall :
        import( AssignmentExpression )

    1. Let referencingScriptOrModule be ! GetActiveScriptOrModule().
    2. Assert: referencingScriptOrModule is a Script Record or Module Record (i.e. is not null).
    3. Let argRef be the result of evaluating AssignmentExpression.
    4. Let specifier be ? GetValue(argRef).
    5. Let promiseCapability be ! NewPromiseCapability(%Promise%).
    6. Let specifierString be ToString(specifier).
    7. IfAbruptRejectPromise(specifierString, promiseCapability).
    8. Perform ! HostImportModuleDynamically(referencingScriptOrModule, specifierString, promiseCapability).
    9. Return promiseCapability.[[Promise]].


    GetBindingValue (N, S)

    [...]
    3. If the binding for N is an indirect binding, then
       a. Let M and N2 be the indirection values provided when this binding for
          N was created.
       b. Let targetEnv be M.[[Environment]].
       c. If targetEnv is undefined, throw a ReferenceError exception.
       d. Let targetER be targetEnv's EnvironmentRecord.
       e. Return ? targetER.GetBindingValue(N2, S).

---*/

if (true) import('./eval-gtbndng-indirect-update-dflt_FIXTURE.js').then(imported => {

  assert.sameValue(imported.default(), 1);
  assert.sameValue(imported.default, 2);

}).then($DONE, $DONE).catch($DONE);
