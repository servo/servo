// This file was procedurally generated from the following sources:
// - src/dynamic-import/file-does-not-exist.case
// - src/dynamic-import/catch/nested-async-arrow-fn-return-await.template
/*---
description: Non existent file can't resolve to a Script or Module Record (nested in async arrow function, returned)
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


    If a Module Record corresponding to the pair referencingModulereferencingScriptOrModule,
    specifier does not exist or cannot be created, an exception must be thrown.

---*/

const f = async () => await import('./THIS_FILE_DOES_NOT_EXIST.js');

f().catch(error => {

  assert.notSameValue(typeof error, 'undefined');

}).then($DONE, $DONE);
