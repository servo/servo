// This file was procedurally generated from the following sources:
// - src/class-elements/escaped-static-static-field-no-asi-before-method.case
// - src/class-elements/default/cls-decl.template
/*---
description: ASI does not separate an escaped static static field from a following method on the same line (field definitions in a class declaration)
esid: prod-FieldDefinition
features: [class-fields-public, class-static-fields-public, class]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
---*/
$DONOTEVALUATE();


class C {
  static st\u0061tic value() {}
}


