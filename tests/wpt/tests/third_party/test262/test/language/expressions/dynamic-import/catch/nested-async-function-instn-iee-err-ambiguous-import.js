// This file was procedurally generated from the following sources:
// - src/dynamic-import/instn-iee-err-ambiguous-import.case
// - src/dynamic-import/catch/nested-async-function.template
/*---
description: IndirectExportEntries validation - ambiguous imported bindings (nested in async function)
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


    [...]
    9. For each ExportEntry Record e in module.[[IndirectExportEntries]], do
       a. Let resolution be ? module.ResolveExport(e.[[ExportName]], « », « »).
       b. If resolution is null or resolution is "ambiguous", throw a
          SyntaxError exception.
    [...]

    15.2.1.16.3 ResolveExport

    [...]
    9. Let starResolution be null.
    10. For each ExportEntry Record e in module.[[StarExportEntries]], do
        a. Let importedModule be ? HostResolveImportedModule(module,
           e.[[ModuleRequest]]).
        b. Let resolution be ? importedModule.ResolveExport(exportName,
           resolveSet, exportStarSet).
        c. If resolution is "ambiguous", return "ambiguous".
        d. If resolution is not null, then
           i. If starResolution is null, let starResolution be resolution.
           ii. Else,
               1. Assert: there is more than one * import that includes the
                  requested name.
               2. If resolution.[[Module]] and starResolution.[[Module]] are
                  not the same Module Record or
                  SameValue(resolution.[[BindingName]],
                  starResolution.[[BindingName]]) is false, return "ambiguous".

---*/

async function f() {
  import('./instn-iee-err-ambiguous-export_FIXTURE.js').catch(error => {

    assert.sameValue(error.name, 'SyntaxError');

  }).then($DONE, $DONE);
}

f();

