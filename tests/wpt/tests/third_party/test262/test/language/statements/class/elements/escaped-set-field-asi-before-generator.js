// This file was procedurally generated from the following sources:
// - src/class-elements/escaped-set-field-asi-before-generator.case
// - src/class-elements/default/cls-decl.template
/*---
description: ASI separates escaped set instance and static fields from following generators (field definitions in a class declaration)
esid: prod-FieldDefinition
features: [class-fields-public, class-static-fields-public, generators, class]
flags: [generated]
includes: [compareArray.js]
---*/


class C {
  \u0073\u0065\u0074
  *a() { yield 1; }
  static \u0073\u0065\u0074
  *b() { yield 2; }
}

var c = new C();
assert.compareArray(Object.keys(c), ["set"]);
assert.compareArray(Object.keys(C), ["set"]);
assert.compareArray(Object.getOwnPropertyNames(C.prototype), ["constructor", "a", "b"]);
assert.compareArray([c.a().next().value, c.b().next().value], [1, 2]);
