// This file was procedurally generated from the following sources:
// - src/class-elements/escaped-async-field-asi-before-generator.case
// - src/class-elements/default/cls-decl.template
/*---
description: ASI separates escaped async instance and static fields from following generators (field definitions in a class declaration)
esid: prod-FieldDefinition
features: [class-fields-public, class-static-fields-public, generators, class]
flags: [generated]
includes: [compareArray.js]
---*/


class C {
  \u0061\u0073\u0079\u006e\u0063
  *a() { yield 1; }
  static \u0061\u0073\u0079\u006e\u0063
  *b() { yield 2; }
}

var c = new C();
assert.compareArray(Object.keys(c), ["async"]);
assert.compareArray(Object.keys(C), ["async"]);
assert.compareArray(Object.getOwnPropertyNames(C.prototype), ["constructor", "a", "b"]);
assert.compareArray([c.a().next().value, c.b().next().value], [1, 2]);
