// This file was procedurally generated from the following sources:
// - src/dynamic-import/typeof-import.case
// - src/dynamic-import/syntax/invalid/top-level.template
/*---
description: It's a SyntaxError if '()' is omitted (top level syntax)
esid: sec-import-call-runtime-semantics-evaluation
features: [dynamic-import]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    ImportCall :
        import( AssignmentExpression )


    ImportCall[Yield, Await] :
        import . source ( AssignmentExpression[+In, ?Yield, ?Await] )
---*/

$DONOTEVALUATE();

typeof import;

/* The params region intentionally empty */
