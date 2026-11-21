// This file was procedurally generated from the following sources:
// - src/class-elements/escaped-get-field-asi-before-field.case
// - src/class-elements/default/cls-expr.template
/*---
description: ASI separates escaped get instance and static fields from following fields (field definitions in a class expression)
esid: prod-FieldDefinition
features: [class-fields-public, class-static-fields-public, class]
flags: [generated]
includes: [compareArray.js]
---*/


var C = class {
  \u0067et
  a = 1;
  static \u0067et
  b = 2;
}

var c = new C();
assert.compareArray(Object.keys(c), ["get", "a", "b"]);
assert.compareArray(Object.keys(C), ["get"]);
assert.compareArray([c.a, c.b], [1, 2]);
