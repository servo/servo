// This file was procedurally generated from the following sources:
// - src/dynamic-import/import-defer-no-new-call-expression-prop-access.case
// - src/dynamic-import/syntax/invalid/top-level.template
/*---
description: ImportCall is a CallExpression, it can't be preceded by the new keyword (property access) (top level syntax)
esid: sec-import-call-runtime-semantics-evaluation
features: [import-defer, dynamic-import]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    ImportCall :
        import( AssignmentExpression )


    CallExpression:
      ImportCall
      CallExpression . IdentifierName

    ImportCall :
        import . defer ( AssignmentExpression[+In, ?Yield, ?Await] )

    NewExpression :
      MemberExpression
      new NewExpression

---*/

$DONOTEVALUATE();

new import.defer('./empty_FIXTURE.js').prop;
