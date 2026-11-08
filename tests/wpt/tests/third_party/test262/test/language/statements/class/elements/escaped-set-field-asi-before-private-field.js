// This file was procedurally generated from the following sources:
// - src/class-elements/escaped-set-field-asi-before-private-field.case
// - src/class-elements/default/cls-decl.template
/*---
description: ASI separates escaped set instance and static fields from following private fields (field definitions in a class declaration)
esid: prod-FieldDefinition
features: [class-fields-public, class-static-fields-public, class-fields-private, class]
flags: [generated]
includes: [compareArray.js]
---*/


class C {
  \u{73}et
  #a = 1;
  static \u{73}et
  #b = 2;
  check() { return [this.#a, this.#b]; }
}

var c = new C();
assert.compareArray(Object.keys(c), ["set"]);
assert.compareArray(Object.keys(C), ["set"]);
assert.compareArray(c.check(), [1, 2]);
