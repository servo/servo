// This file was procedurally generated from the following sources:
// - src/class-elements/escaped-get-field-asi-end.case
// - src/class-elements/default/cls-decl.template
/*---
description: ASI after an escaped get field at the end of a class body (field definitions in a class declaration)
esid: prod-FieldDefinition
features: [class-fields-public, class]
flags: [generated]
includes: [compareArray.js]
---*/


class C {
  ge\u0074
}

var c = new C();
assert.compareArray(Object.keys(c), ["get"]);
assert.compareArray(Object.keys(C), []);
