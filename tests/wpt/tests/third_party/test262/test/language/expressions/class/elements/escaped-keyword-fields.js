// This file was procedurally generated from the following sources:
// - src/class-elements/escaped-keyword-fields.case
// - src/class-elements/default/cls-expr.template
/*---
description: Escaped contextual keywords as instance and static field names (field definitions in a class expression)
esid: prod-FieldDefinition
features: [class-fields-public, class-static-fields-public, class]
flags: [generated]
includes: [compareArray.js]
---*/


var C = class {
  \u0067et;
  \u0073et;
  \u0073tatic;
  \u0061sync;
  \u0061ccessor;
  static \u0067et;
  static \u0073et;
  static \u0073tatic;
  static \u0061sync;
  static \u0061ccessor;
}

var c = new C();
assert.compareArray(Object.keys(c), ["get", "set", "static", "async", "accessor"]);
assert.compareArray(Object.keys(C), ["get", "set", "static", "async", "accessor"]);
assert.compareArray([c.get, c.set, c.static, c.async, c.accessor], [undefined, undefined, undefined, undefined, undefined]);
assert.compareArray([C.get, C.set, C.static, C.async, C.accessor], [undefined, undefined, undefined, undefined, undefined]);
