// This file was procedurally generated from the following sources:
// - src/class-elements/escaped-accessor-field-asi-end.case
// - src/class-elements/default/cls-expr.template
/*---
description: ASI after an escaped accessor field at the end of a class body (field definitions in a class expression)
esid: prod-FieldDefinition
features: [class-fields-public, class]
flags: [generated]
includes: [compareArray.js]
---*/


var C = class {
  accesso\u0072
}

var c = new C();
assert.compareArray(Object.keys(c), ["accessor"]);
assert.compareArray(Object.keys(C), []);
