// This file was procedurally generated from the following sources:
// - src/dynamic-import/import-source-specifier-tostring-abrupt-rejects.case
// - src/dynamic-import/catch/nested-function.template
/*---
description: Abrupt from ToString(specifier) rejects the promise (nested function)
esid: sec-import-call-runtime-semantics-evaluation
features: [source-phase-imports, dynamic-import]
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

    ImportCall : import . source ( AssignmentExpression )
    1. Return ? EvaluateImportCall(AssignmentExpression, source).

    13.3.10.1.1 EvaluateImportCall ( specifierExpression, phase )
    1. Let referrer be GetActiveScriptOrModule().
    2. If referrer is null, set referrer to the current Realm Record.
    3. Let specifierRef be ? Evaluation of specifierExpression.
    4. Let specifier be ? GetValue(specifierRef).
    5. Let promiseCapability be ! NewPromiseCapability(%Promise%).
    6. Let specifierString be Completion(ToString(specifier)).
    7. IfAbruptRejectPromise(specifierString, promiseCapability).
    8. Let moduleRequest be a new ModuleRequest Record { [[Specifier]]: specifierString, [[Phase]]: phase }.
    9. Perform HostLoadImportedModule(referrer, moduleRequest, empty, promiseCapability).
    10. Return promiseCapability.[[Promise]].

---*/
const obj = {
    toString() {
        throw 'custom error';
    }
};


function f() {
  import.source(obj).catch(error => {

    assert.sameValue(error, 'custom error');

  }).then($DONE, $DONE);
}
f();
