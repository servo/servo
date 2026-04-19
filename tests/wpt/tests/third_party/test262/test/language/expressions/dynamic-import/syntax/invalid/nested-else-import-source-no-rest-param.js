// This file was procedurally generated from the following sources:
// - src/dynamic-import/import-source-no-rest-param.case
// - src/dynamic-import/syntax/invalid/nested-else.template
/*---
description: ImportCall is not extensible - no rest parameter (nested else syntax)
esid: sec-import-call-runtime-semantics-evaluation
features: [source-phase-imports, source-phase-imports-module-source, dynamic-import]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
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


    ImportCall :
        import . source ( AssignmentExpression[+In, ?Yield] )

    Forbidden Extensions

    - ImportCall must not be extended.

    This production doesn't allow the following production from ArgumentsList:

    ... AssignmentExpression

---*/

$DONOTEVALUATE();

if (false) {

} else {
  import.source(...['<module source>']);
}
