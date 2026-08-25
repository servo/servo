// This file was procedurally generated from the following sources:
// - src/dynamic-import/import-defer-no-rest-param.case
// - src/dynamic-import/syntax/invalid/top-level.template
/*---
description: ImportCall is not extensible - no rest parameter (top level syntax)
esid: sec-import-call-runtime-semantics-evaluation
features: [import-defer, dynamic-import]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    ImportCall :
        import( AssignmentExpression )


    ImportCall :
        import . defer ( AssignmentExpression[+In, ?Yield] )

    This production doesn't allow the following production from ArgumentsList:

    ... AssignmentExpression

---*/

$DONOTEVALUATE();

import.defer(...['./empty_FIXTURE.js']);
