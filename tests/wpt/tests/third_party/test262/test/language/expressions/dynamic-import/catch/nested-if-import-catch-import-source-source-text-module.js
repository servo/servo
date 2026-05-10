// This file was procedurally generated from the following sources:
// - src/dynamic-import/import-source-source-text-module.case
// - src/dynamic-import/catch/nested-if.template
/*---
description: GetModuleSource of SourceTextModule always returns an abrupt completion. (nested if)
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


    16.2.1.7.2 GetModuleSource ( )
    Source Text Module Record provides a GetModuleSource implementation that always returns an abrupt completion indicating that a source phase import is not available.
    1. Throw a SyntaxError exception.

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

if (true) {
  import.source('./empty_FIXTURE.js').catch(error => {

    assert.sameValue(error.name, 'SyntaxError');

  }).then($DONE, $DONE);
}
