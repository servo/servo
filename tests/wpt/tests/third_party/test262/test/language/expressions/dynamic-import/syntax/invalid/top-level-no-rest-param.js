// This file was procedurally generated from the following sources:
// - src/dynamic-import/no-rest-param.case
// - src/dynamic-import/syntax/invalid/top-level.template
/*---
description: ImportCall is not extensible - no rest parameter (top level syntax)
esid: sec-import-call-runtime-semantics-evaluation
features: [dynamic-import]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    ImportCall :
        import( AssignmentExpression )


    ImportCall :
        import( AssignmentExpression[+In, ?Yield] )

    Forbidden Extensions

    - ImportCall must not be extended.

    This production doesn't allow the following production from ArgumentsList:

    ... AssignmentExpression
---*/

$DONOTEVALUATE();

import(...['']);
