// This file was procedurally generated from the following sources:
// - src/class-elements/escaped-async-field-asi-end.case
// - src/class-elements/default/cls-decl.template
/*---
description: ASI after an escaped async field at the end of a class body (field definitions in a class declaration)
esid: prod-FieldDefinition
features: [class-fields-public, class]
flags: [generated]
includes: [compareArray.js]
---*/


class C {
  asyn\u0063
}

var c = new C();
assert.compareArray(Object.keys(c), ["async"]);
assert.compareArray(Object.keys(C), []);
