// This file was procedurally generated from the following sources:
// - src/class-elements/escaped-get-field-asi-before-method.case
// - src/class-elements/default/cls-expr.template
/*---
description: ASI separates escaped get instance and static fields from following methods (field definitions in a class expression)
esid: prod-FieldDefinition
features: [class-fields-public, class-static-fields-public, class]
flags: [generated]
includes: [compareArray.js]
---*/


var C = class {
  g\u0065t
  a() { return 1; }
  static g\u0065t
  b() { return 2; }
}

var c = new C();
assert.compareArray(Object.keys(c), ["get"]);
assert.compareArray(Object.keys(C), ["get"]);
assert.compareArray(Object.getOwnPropertyNames(C.prototype), ["constructor", "a", "b"]);
assert.compareArray([c.a(), c.b()], [1, 2]);
