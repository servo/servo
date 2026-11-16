// This file was procedurally generated from the following sources:
// - src/class-elements/escaped-async-field-asi-before-computed-field.case
// - src/class-elements/default/cls-expr.template
/*---
description: ASI separates escaped async instance and static fields from following computed fields (field definitions in a class expression)
esid: prod-FieldDefinition
features: [class-fields-public, class-static-fields-public, computed-property-names, class]
flags: [generated]
includes: [compareArray.js]
---*/


var C = class {
  asyn\u{63}
  ["a"] = 1;
  static asyn\u{63}
  ["b"] = 2;
}

var c = new C();
assert.compareArray(Object.keys(c), ["async", "a", "b"]);
assert.compareArray(Object.keys(C), ["async"]);
assert.compareArray([c.a, c.b], [1, 2]);
