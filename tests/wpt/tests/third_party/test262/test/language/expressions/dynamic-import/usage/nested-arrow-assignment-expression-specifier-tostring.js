// This file was procedurally generated from the following sources:
// - src/dynamic-import/specifier-tostring.case
// - src/dynamic-import/default/nested-arrow-assign-expr.template
/*---
description: ToString value of specifier (nested arrow)
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


    Import Calls

    Runtime Semantics: Evaluation

    ImportCall : import(AssignmentExpression)

    1. Let referencingScriptOrModule be ! GetActiveScriptOrModule().
    2. Let argRef be the result of evaluating AssignmentExpression.
    3. Let specifier be ? GetValue(argRef).
    4. Let promiseCapability be ! NewPromiseCapability(%Promise%).
    5. Let specifierString be ToString(specifier).
    6. IfAbruptRejectPromise(specifierString, promiseCapability).
    7. Perform ! HostImportModuleDynamically(referencingScriptOrModule, specifierString, promiseCapability).
    8. Return promiseCapability.[[Promise]].

---*/
// import('./module-code_FIXTURE.js')

const obj = {
    toString() {
        return './module-code_FIXTURE.js';
    }
};


let f = () => import(obj).then(imported => {

  assert.sameValue(imported.default, 42);
  assert.sameValue(imported.x, 'Test262');
  assert.sameValue(imported.z, 42);

});


f().then($DONE, $DONE).catch($DONE);
