// This file was procedurally generated from the following sources:
// - src/class-elements/escaped-keyword-fields-initializer.case
// - src/class-elements/default/cls-decl.template
/*---
description: Escaped contextual keywords as instance and static field names with initializers (field definitions in a class declaration)
esid: prod-FieldDefinition
features: [class-fields-public, class-static-fields-public, class]
flags: [generated]
includes: [compareArray.js]
---*/


class C {
  \u{67}et = 1;
  \u{73}et = 2;
  \u{73}tatic = 3;
  \u{61}sync = 4;
  \u{61}ccessor = 5;
  static \u{67}et = 1;
  static \u{73}et = 2;
  static \u{73}tatic = 3;
  static \u{61}sync = 4;
  static \u{61}ccessor = 5;
}

var c = new C();
assert.compareArray(Object.keys(c), ["get", "set", "static", "async", "accessor"]);
assert.compareArray(Object.keys(C), ["get", "set", "static", "async", "accessor"]);
assert.compareArray([c.get, c.set, c.static, c.async, c.accessor], [1, 2, 3, 4, 5]);
assert.compareArray([C.get, C.set, C.static, C.async, C.accessor], [1, 2, 3, 4, 5]);
