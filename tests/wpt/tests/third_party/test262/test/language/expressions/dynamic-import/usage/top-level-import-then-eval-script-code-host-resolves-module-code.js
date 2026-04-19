// This file was procedurally generated from the following sources:
// - src/dynamic-import/eval-script-code-host-resolves-module-code.case
// - src/dynamic-import/default/top-level.template
/*---
description: import() from a script code can load a file with module code (top level)
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
// This is still valid in script code, and should not be valid for module code
// https://tc39.github.io/ecma262/#sec-scripts-static-semantics-lexicallydeclarednames
var smoosh; function smoosh() {}


import('./module-code_FIXTURE.js').then(imported => {

  assert.sameValue(imported.default, 42);
  assert.sameValue(imported.x, 'Test262');
  assert.sameValue(imported.z, 42);

}).then($DONE, $DONE).catch($DONE);
