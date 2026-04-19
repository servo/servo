// This file was procedurally generated from the following sources:
// - src/dynamic-import/import-call-unknown.case
// - src/dynamic-import/syntax/invalid/top-level.template
/*---
description: It's a SyntaxError on unknown import call (top level syntax)
esid: sec-import-call-runtime-semantics-evaluation
features: [source-phase-imports, dynamic-import]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    ImportCall :
        import( AssignmentExpression )


    ImportCall[Yield, Await] :
        import ( AssignmentExpression[+In, ?Yield, ?Await] )
        import . source ( AssignmentExpression[+In, ?Yield, ?Await] )

---*/

$DONOTEVALUATE();

import.UNKNOWN('./empty_FIXTURE.js');

/* The params region intentionally empty */
