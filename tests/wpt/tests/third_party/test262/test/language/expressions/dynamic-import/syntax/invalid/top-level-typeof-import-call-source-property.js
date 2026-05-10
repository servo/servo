// This file was procedurally generated from the following sources:
// - src/dynamic-import/typeof-import-call-source-property.case
// - src/dynamic-import/syntax/invalid/top-level.template
/*---
description: It's a SyntaxError on unexpected import source property (top level syntax)
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

typeof import.source.UNKNOWN;

/* The params region intentionally empty */
