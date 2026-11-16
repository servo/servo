// This file was procedurally generated from the following sources:
// - src/class-elements/escaped-accessor-field-asi-before-generator.case
// - src/class-elements/default/cls-decl.template
/*---
description: ASI separates escaped accessor instance and static fields from following generators (field definitions in a class declaration)
esid: prod-FieldDefinition
features: [class-fields-public, class-static-fields-public, generators, class]
flags: [generated]
includes: [compareArray.js]
---*/


class C {
  \u0061\u0063\u0063\u0065\u0073\u0073\u006f\u0072
  *a() { yield 1; }
  static \u0061\u0063\u0063\u0065\u0073\u0073\u006f\u0072
  *b() { yield 2; }
}

var c = new C();
assert.compareArray(Object.keys(c), ["accessor"]);
assert.compareArray(Object.keys(C), ["accessor"]);
assert.compareArray(Object.getOwnPropertyNames(C.prototype), ["constructor", "a", "b"]);
assert.compareArray([c.a().next().value, c.b().next().value], [1, 2]);
