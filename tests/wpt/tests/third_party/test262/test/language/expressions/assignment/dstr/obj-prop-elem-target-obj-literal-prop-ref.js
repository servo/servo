// This file was procedurally generated from the following sources:
// - src/dstr-assignment/obj-prop-elem-target-obj-literal-prop-ref.case
// - src/dstr-assignment/default/assignment-expr.template
/*---
description: The DestructuringAssignmentTarget of an AssignmentElement can extend to LHSExpressions if it is neither an ObjectLiteral nor an ArrayLiteral and its AssignmentTargetTyp is simple. Using MemberExpression (ObjLiteral + identifier). (AssignmentExpression)
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

    AssignmentPattern : ObjectAssignmentPattern
    ObjectAssignmentPattern : { AssignmentPropertyList }
    AssignmentPropertyList : AssignmentProperty
    AssignmentProperty : PropertyName : AssignmentElement
    AssignmentElement : DestructuringAssignmentTarget Initializer_opt
    DestructuringAssignmentTarget : LeftHandSideExpression

    Static Semantics: Early Errors

    DestructuringAssignmentTarget : LeftHandSideExpression

    - It is a Syntax Error if LeftHandSideExpression is either an ObjectLiteral or an ArrayLiteral and if LeftHandSideExpression is not covering an AssignmentPattern.
    - It is a Syntax Error if LeftHandSideExpression is neither an ObjectLiteral nor an ArrayLiteral and AssignmentTargetType(LeftHandSideExpression) is not simple.

    Runtime Semantics: DestructuringAssignmentEvaluation
    ObjectAssignmentPattern : { AssignmentPropertyList }

    1. Perform ? RequireObjectCoercible(value).
    2. Perform ? PropertyDestructuringAssignmentEvaluation for AssignmentPropertyList using value as the argument.
    3. Return NormalCompletion(empty).

    Runtime Semantics: PropertyDestructuringAssignmentEvaluation

    AssignmentProperty : PropertyName : AssignmentElement

    1. Let name be the result of evaluating PropertyName.
    2. ReturnIfAbrupt(name).
    3. Perform ? KeyedDestructuringAssignmentEvaluation of AssignmentElement with value and name as the arguments.
    4. Return a new List containing name.

    Runtime Semantics: KeyedDestructuringAssignmentEvaluation

    AssignmentElement : DestructuringAssignmentTarget Initializer

    1. If DestructuringAssignmentTarget is neither an ObjectLiteral nor an ArrayLiteral, then
      a. Let lref be the result of evaluating DestructuringAssignmentTarget.
    ...

---*/
var setValue;

var result;
var vals = {x: 23};

result = { x: {
  get y() {
    throw new Test262Error('The property should not be accessed.');
  },
  set y(val) {
    setValue = val;
  }
}.y} = vals;

assert.sameValue(setValue, 23);


assert.sameValue(result, vals);
