// This file was procedurally generated from the following sources:
// - src/class-elements/escaped-accessor-field-asi-before-field.case
// - src/class-elements/default/cls-decl.template
/*---
description: ASI separates escaped accessor instance and static fields from following fields (field definitions in a class declaration)
esid: prod-FieldDefinition
features: [class-fields-public, class-static-fields-public, class]
flags: [generated]
includes: [compareArray.js]
---*/


class C {
  \u0061ccessor
  a = 1;
  static \u0061ccessor
  b = 2;
}

var c = new C();
assert.compareArray(Object.keys(c), ["accessor", "a", "b"]);
assert.compareArray(Object.keys(C), ["accessor"]);
assert.compareArray([c.a, c.b], [1, 2]);
