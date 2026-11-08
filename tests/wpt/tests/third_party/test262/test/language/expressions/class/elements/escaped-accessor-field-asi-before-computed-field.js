// This file was procedurally generated from the following sources:
// - src/class-elements/escaped-accessor-field-asi-before-computed-field.case
// - src/class-elements/default/cls-expr.template
/*---
description: ASI separates escaped accessor instance and static fields from following computed fields (field definitions in a class expression)
esid: prod-FieldDefinition
features: [class-fields-public, class-static-fields-public, computed-property-names, class]
flags: [generated]
includes: [compareArray.js]
---*/


var C = class {
  accesso\u{72}
  ["a"] = 1;
  static accesso\u{72}
  ["b"] = 2;
}

var c = new C();
assert.compareArray(Object.keys(c), ["accessor", "a", "b"]);
assert.compareArray(Object.keys(C), ["accessor"]);
assert.compareArray([c.a, c.b], [1, 2]);
