// This file was procedurally generated from the following sources:
// - src/class-elements/escaped-async-field-asi-before-private-field.case
// - src/class-elements/default/cls-decl.template
/*---
description: ASI separates escaped async instance and static fields from following private fields (field definitions in a class declaration)
esid: prod-FieldDefinition
features: [class-fields-public, class-static-fields-public, class-fields-private, class]
flags: [generated]
includes: [compareArray.js]
---*/


class C {
  \u{61}sync
  #a = 1;
  static \u{61}sync
  #b = 2;
  check() { return [this.#a, this.#b]; }
}

var c = new C();
assert.compareArray(Object.keys(c), ["async"]);
assert.compareArray(Object.keys(C), ["async"]);
assert.compareArray(c.check(), [1, 2]);
