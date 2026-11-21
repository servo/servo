// This file was procedurally generated from the following sources:
// - src/class-elements/escaped-set-static-field-no-asi-before-method.case
// - src/class-elements/default/cls-expr.template
/*---
description: ASI does not separate an escaped set static field from a following method on the same line (field definitions in a class expression)
esid: prod-FieldDefinition
features: [class-fields-public, class-static-fields-public, class]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
---*/
$DONOTEVALUATE();


var C = class {
  static s\u0065t value(v) {}
}


