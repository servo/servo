// This file was procedurally generated from the following sources:
// - src/class-elements/escaped-async-field-asi-before-comment.case
// - src/class-elements/default/cls-decl.template
/*---
description: ASI separates escaped async instance and static fields from following fields separated by multiline comments (field definitions in a class declaration)
esid: prod-FieldDefinition
features: [class-fields-public, class-static-fields-public, class]
flags: [generated]
includes: [compareArray.js]
---*/


class C {
  a\u0073ync /*
  */ a = 1;
  static a\u0073ync /*
  */ b = 2;
}

var c = new C();
assert.compareArray(Object.keys(c), ["async", "a", "b"]);
assert.compareArray(Object.keys(C), ["async"]);
assert.compareArray([c.a, c.b], [1, 2]);
