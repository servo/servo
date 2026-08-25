// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-id-identifier-yield-ident-invalid.case
// - src/dstr-assignment/syntax/for-of.template
/*---
description: yield is not a valid IdentifierReference in an AssignmentProperty within strict mode code. (For..of statement)
esid: sec-for-in-and-for-of-statements-runtime-semantics-labelledevaluation
features: [destructuring-binding]
flags: [generated, onlyStrict]
negative:
  phase: parse
  type: SyntaxError
info: |
    IterationStatement :
      for ( LeftHandSideExpression of AssignmentExpression ) Statement

    1. Let keyResult be the result of performing ? ForIn/OfHeadEvaluation(« »,
       AssignmentExpression, iterate).
    2. Return ? ForIn/OfBodyEvaluation(LeftHandSideExpression, Statement,
       keyResult, assignment, labelSet).

    13.7.5.13 Runtime Semantics: ForIn/OfBodyEvaluation

    [...]
    4. If destructuring is true and if lhsKind is assignment, then
       a. Assert: lhs is a LeftHandSideExpression.
       b. Let assignmentPattern be the parse of the source text corresponding to
          lhs using AssignmentPattern as the goal symbol.
    [...]
---*/
$DONOTEVALUATE();

for ({ yield } of [{}]) ;
