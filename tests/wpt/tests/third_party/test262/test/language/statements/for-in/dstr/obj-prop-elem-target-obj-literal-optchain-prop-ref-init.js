// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-prop-elem-target-obj-literal-optchain-prop-ref-init.case
// - src/dstr-assignment/syntax/for-in.template
/*---
description: It is a Syntax Error if LeftHandSideExpression of an DestructuringAssignmentTarget is neither an ObjectLiteral nor an ArrayLiteral and AssignmentTargetType(LeftHandSideExpression) is not simple Using Object (For..in statement)
esid: sec-for-in-and-for-of-statements-runtime-semantics-labelledevaluation
features: [optional-chaining, destructuring-binding]
flags: [generated]
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

    Syntax

    AssignmentElement : DestructuringAssignmentTarget Initializer_opt
    DestructuringAssignmentTarget : LeftHandSideExpression

    Static Semantics: Early Errors

    OptionalExpression:
      MemberExpression OptionalChain
      CallExpression OptionalChain
      OptionalExpression OptionalChain

    OptionalChain:
      ?. [ Expression ]
      ?. IdentifierName
      ?. Arguments 
      ?. TemplateLiteral
      OptionalChain [ Expression ]
      OptionalChain .IdentifierName
      OptionalChain Arguments 
      OptionalChain TemplateLiteral

    DestructuringAssignmentTarget : LeftHandSideExpression

    - It is a Syntax Error if LeftHandSideExpression is neither an ObjectLiteral nor an ArrayLiteral and IsValidSimpleAssignmentTarget(LeftHandSideExpression) is not true.

    Static Semantics: IsValidSimpleAssignmentTarget

    LeftHandSideExpression : OptionalExpression
      1. Return false.

---*/
$DONOTEVALUATE();

for ({ x: {
  set y(val) {
    throw new Test262Error('The property should not be accessed.');
  }
}?.y = 42} in [{x: 42}]) ;
