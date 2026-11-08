// This file was procedurally generated from the following sources:
// - src/class-elements/escaped-set-field-asi-before-method.case
// - src/class-elements/default/cls-decl.template
/*---
description: ASI separates escaped set instance and static fields from following methods (field definitions in a class declaration)
esid: prod-FieldDefinition
features: [class-fields-public, class-static-fields-public, class]
flags: [generated]
includes: [compareArray.js]
---*/


class C {
  s\u0065t
  a() { return 1; }
  static s\u0065t
  b() { return 2; }
}

var c = new C();
assert.compareArray(Object.keys(c), ["set"]);
assert.compareArray(Object.keys(C), ["set"]);
assert.compareArray(Object.getOwnPropertyNames(C.prototype), ["constructor", "a", "b"]);
assert.compareArray([c.a(), c.b()], [1, 2]);
