// This file was procedurally generated from the following sources:
// - src/class-elements/escaped-set-field-asi-before-comment.case
// - src/class-elements/default/cls-expr.template
/*---
description: ASI separates escaped set instance and static fields from following fields separated by multiline comments (field definitions in a class expression)
esid: prod-FieldDefinition
features: [class-fields-public, class-static-fields-public, class]
flags: [generated]
includes: [compareArray.js]
---*/


var C = class {
  s\u0065t /*
  */ a = 1;
  static s\u0065t /*
  */ b = 2;
}

var c = new C();
assert.compareArray(Object.keys(c), ["set", "a", "b"]);
assert.compareArray(Object.keys(C), ["set"]);
assert.compareArray([c.a, c.b], [1, 2]);
