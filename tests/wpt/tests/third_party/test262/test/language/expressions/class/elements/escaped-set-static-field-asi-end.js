// This file was procedurally generated from the following sources:
// - src/class-elements/escaped-set-static-field-asi-end.case
// - src/class-elements/default/cls-expr.template
/*---
description: ASI after an escaped set static field at the end of a class body (field definitions in a class expression)
esid: prod-FieldDefinition
features: [class-fields-public, class-static-fields-public, class]
flags: [generated]
includes: [compareArray.js]
---*/


var C = class {
  static se\u0074
}

var c = new C();
assert.compareArray(Object.keys(c), []);
assert.compareArray(Object.keys(C), ["set"]);
