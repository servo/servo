// This file was procedurally generated from the following sources:
// - src/dynamic-import/eval-script-code-target.case
// - src/dynamic-import/catch/nested-async-generator-return-await.template
/*---
description: import() from a module code can load a file with script code, but the target is resolved into a Module Record (nested in async generator, returns awaited)
esid: sec-import-call-runtime-semantics-evaluation
features: [dynamic-import, async-iteration]
flags: [generated, module, async]
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


    Modules

    Static Semantics: Early Errors

      ModuleBody : ModuleItemList
      - It is a Syntax Error if the LexicallyDeclaredNames of ModuleItemList containsany duplicate entries.
      - It is a Syntax Error if any element of the LexicallyDeclaredNames of ModuleItemList also occurs in the VarDeclaredNames of ModuleItemList.

---*/

async function * f() {
  return await import('./script-code_FIXTURE.js');
}

f().next().catch(error => {

  assert.sameValue(error.name, 'SyntaxError');

}).then($DONE, $DONE);
