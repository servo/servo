// This file was procedurally generated from the following sources:
// - src/dynamic-import/import-source-no-rest-param.case
// - src/dynamic-import/syntax/invalid/top-level.template
/*---
description: ImportCall is not extensible - no rest parameter (top level syntax)
esid: sec-import-call-runtime-semantics-evaluation
features: [source-phase-imports, source-phase-imports-module-source, dynamic-import]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    ImportCall :
        import( AssignmentExpression )


    ImportCall :
        import . source ( AssignmentExpression[+In, ?Yield] )

    Forbidden Extensions

    - ImportCall must not be extended.

    This production doesn't allow the following production from ArgumentsList:

    ... AssignmentExpression

---*/

$DONOTEVALUATE();

import.source(...['<module source>']);
