// This file was procedurally generated from the following sources:
// - src/class-elements/grammar-private-field-on-object-destructuring.case
// - src/class-elements/syntax/invalid/cls-expr-elements-invalid-syntax.template
/*---
description: Acessing private field from object destructuring pattern is not a valid syntax (class expression)
esid: prod-ClassElement
features: [class-fields-private, destructuring-binding, class]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Updated Productions

    ObjectAssignmentPattern[Yield, Await]:
       {}
       {AssignmentRestProperty[?Yield, ?Await]}
       {AssignmentPropertyList[?Yield, ?Await]}
       {AssignmentPropertyList[?Yield, ?Await],AssignmentRestProperty[?Yield, ?Await]opt}

---*/


$DONOTEVALUATE();

var C = class {
  #x = 1;

  destructureX() {
    const { #x: x } = this;
  }
};
