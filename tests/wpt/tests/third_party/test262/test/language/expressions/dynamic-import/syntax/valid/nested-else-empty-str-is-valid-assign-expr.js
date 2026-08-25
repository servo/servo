// This file was procedurally generated from the following sources:
// - src/dynamic-import/empty-str-is-valid-assign-expr.case
// - src/dynamic-import/syntax/valid/nested-else.template
/*---
description: Calling import('') (nested else syntax)
esid: sec-import-call-runtime-semantics-evaluation
features: [dynamic-import]
flags: [generated]
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

if (false) {

} else {
  import('');
}
