// This file was procedurally generated from the following sources:
// - src/dynamic-import/not-extensible-args.case
// - src/dynamic-import/syntax/invalid/top-level.template
/*---
description: ImportCall is not extensible - no arguments list (top level syntax)
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
        import( AssignmentExpression[+In, ?Yield, ?Await] ,opt )
        import( AssignmentExpression[+In, ?Yield, ?Await] , AssignmentExpression[+In, ?Yield, ?Await] ,opt )

    Forbidden Extensions

    - ImportCall must not be extended.
---*/

$DONOTEVALUATE();

import('./empty_FIXTURE.js', {}, '');
