// This file was procedurally generated from the following sources:
// - src/class-elements/escaped-get-field-asi-before-computed-field.case
// - src/class-elements/default/cls-expr.template
/*---
description: ASI separates escaped get instance and static fields from following computed fields (field definitions in a class expression)
esid: prod-FieldDefinition
features: [class-fields-public, class-static-fields-public, computed-property-names, class]
flags: [generated]
includes: [compareArray.js]
---*/


var C = class {
  ge\u{74}
  ["a"] = 1;
  static ge\u{74}
  ["b"] = 2;
}

var c = new C();
assert.compareArray(Object.keys(c), ["get", "a", "b"]);
assert.compareArray(Object.keys(C), ["get"]);
assert.compareArray([c.a, c.b], [1, 2]);
