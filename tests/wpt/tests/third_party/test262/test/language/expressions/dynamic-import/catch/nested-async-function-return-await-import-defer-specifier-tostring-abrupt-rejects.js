// This file was procedurally generated from the following sources:
// - src/dynamic-import/import-defer-specifier-tostring-abrupt-rejects.case
// - src/dynamic-import/catch/nested-async-function-return-await.template
/*---
description: Abrupt from ToString(specifier) rejects the promise (nested in async function, returns awaited)
esid: sec-import-call-runtime-semantics-evaluation
features: [import-defer, dynamic-import]
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

    ImportCall : import . defer ( |AssignmentExpression| )
        1. Return ? EvaluateImportCall(|AssignmentExpression|, ~defer~)

    EvaluateImportCall ( specifierExpression, phase )
        1. Let _referrer_ be GetActiveScriptOrModule().
        1. If _referrer_ is *null*, set _referrer_ to the current Realm Record.
        1. Let _specifierRef_ be ? Evaluation of _specifierExpression_.
        1. Let _specifier_ be ? GetValue(_specifierRef_).
        1. Let _promiseCapability_ be ! NewPromiseCapability(%Promise%).
        1. Let _specifierString_ be Completion(ToString(_specifier_)).
        1. IfAbruptRejectPromise(_specifierString_, _promiseCapability_).
        ...

---*/
const obj = {
    toString() {
        throw 'custom error';
    }
};


async function f() {
  return await import.defer(obj).catch(error => {

    assert.sameValue(error, 'custom error');

  }).then($DONE, $DONE);
}

f();
