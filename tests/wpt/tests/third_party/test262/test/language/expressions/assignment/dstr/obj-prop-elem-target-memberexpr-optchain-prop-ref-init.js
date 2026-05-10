// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-prop-elem-target-memberexpr-optchain-prop-ref-init.case
// - src/dstr-assignment/syntax/assignment-expr.template
/*---
description: It is a Syntax Error if LeftHandSideExpression of an DestructuringAssignmentTarget is neither an ObjectLiteral nor an ArrayLiteral and AssignmentTargetType(LeftHandSideExpression) is not simple Using Object (MemberExpression OptionalChain .IdentifierName Initializer) (AssignmentExpression)
esid: sec-variable-statement-runtime-semantics-evaluation
features: [optional-chaining, destructuring-binding]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    VariableDeclaration : BindingPattern Initializer

    1. Let rhs be the result of evaluating Initializer.
    2. Let rval be GetValue(rhs).
    3. ReturnIfAbrupt(rval).
    4. Return the result of performing BindingInitialization for
       BindingPattern passing rval and undefined as arguments.

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
var y = {};

0, { x: y?.z = 42 } = { x: 23 };
