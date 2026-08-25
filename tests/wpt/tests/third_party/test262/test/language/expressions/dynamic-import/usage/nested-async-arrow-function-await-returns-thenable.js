// This file was procedurally generated from the following sources:
// - src/dynamic-import/returns-thenable.case
// - src/dynamic-import/default/nested-async-arrow-fn-await.template
/*---
description: Dynamic import() returns a thenable object. (nested in async arrow function, awaited)
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

---*/

const f = async () => {
  await import('./dynamic-import-module_FIXTURE.js').then(imported => {

    assert.sameValue(imported.x, 1);

  });
}

f().then($DONE, $DONE).catch($DONE);
