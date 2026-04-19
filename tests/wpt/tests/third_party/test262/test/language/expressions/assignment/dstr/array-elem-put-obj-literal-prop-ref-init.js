// This file was procedurally generated from the following sources:
// - src/dstr-assignment/array-elem-put-obj-literal-prop-ref-init.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: The DestructuringAssignmentTarget of an AssignmentElement can extend to LHSExpressions if it is neither an ObjectLiteral nor an ArrayLiteral and its AssignmentTargetTyp is simple. Using MemberExpression (ObjLiteral + identifier) with initializer. (AssignmentExpression)
esid: sec-variable-statement-runtime-semantics-evaluation
features: [destructuring-binding]
flags: [generated]
info: |
    VariableDeclaration : BindingPattern Initializer

    1. Let rhs be the result of evaluating Initializer.
    2. Let rval be GetValue(rhs).
    3. ReturnIfAbrupt(rval).
    4. Return the result of performing BindingInitialization for
       BindingPattern passing rval and undefined as arguments.

    Syntax

    AssignmentPattern : ArrayAssignmentPattern
    ArrayAssignmentPattern : [ AssignmentElementList ]
    AssignmentElementList : AssignmentElisionElement
    AssignmentElisionElement : Elision_opt AssignmentElement
    AssignmentElement : DestructuringAssignmentTarget Initializer_opt
    DestructuringAssignmentTarget : LeftHandSideExpression

    Static Semantics: Early Errors

    DestructuringAssignmentTarget : LeftHandSideExpression

    - It is a Syntax Error if LeftHandSideExpression is either an ObjectLiteral or an ArrayLiteral and if LeftHandSideExpression is not covering an AssignmentPattern.
    - It is a Syntax Error if LeftHandSideExpression is neither an ObjectLiteral nor an ArrayLiteral and AssignmentTargetType(LeftHandSideExpression) is not simple.

    Runtime Semantics: DestructuringAssignmentEvaluation
    ArrayAssignmentPattern : [ AssignmentElementList ]

    1. Let iteratorRecord be ? GetIterator(value).
    2. Let result be IteratorDestructuringAssignmentEvaluation of AssignmentElementList with argument iteratorRecord.
    3. If iteratorRecord.[[Done]] is false, return ? IteratorClose(iteratorRecord, result).
    4. Return result.

    Runtime Semantics: IteratorDestructuringAssignmentEvaluation
    AssignmentElement : DestructuringAssignmentTarget Initializer

    1. If DestructuringAssignmentTarget is neither an ObjectLiteral nor an ArrayLiteral, then
      a. Let lref be the result of evaluating DestructuringAssignmentTarget.
    ...
    7. Return ? PutValue(lref, v).

---*/
var x, setValue;

var result;
var vals = [23];

result = [{
  get y() {
    throw new Test262Error('The property should not be accessed.');
  },
  set y(val) {
    setValue = val;
  }
}.y = 42] = vals;

assert.sameValue(setValue, 23);


assert.sameValue(result, vals);
