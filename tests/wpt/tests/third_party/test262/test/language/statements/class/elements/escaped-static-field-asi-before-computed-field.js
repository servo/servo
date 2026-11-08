// This file was procedurally generated from the following sources:
// - src/class-elements/escaped-static-field-asi-before-computed-field.case
// - src/class-elements/default/cls-decl.template
/*---
description: ASI separates escaped static instance and static fields from following computed fields (field definitions in a class declaration)
esid: prod-FieldDefinition
features: [class-fields-public, class-static-fields-public, computed-property-names, class]
flags: [generated]
includes: [compareArray.js]
---*/


class C {
  stati\u{63}
  ["a"] = 1;
  static stati\u{63}
  ["b"] = 2;
}

var c = new C();
assert.compareArray(Object.keys(c), ["static", "a", "b"]);
assert.compareArray(Object.keys(C), ["static"]);
assert.compareArray([c.a, c.b], [1, 2]);
