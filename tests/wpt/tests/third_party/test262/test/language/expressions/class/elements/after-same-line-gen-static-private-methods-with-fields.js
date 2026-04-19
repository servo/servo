// This file was procedurally generated from the following sources:
// - src/class-elements/static-private-methods-with-fields.case
// - src/class-elements/productions/cls-expr-after-same-line-gen.template
/*---
description: static private methods with fields (field definitions after a generator in the same line)
esid: prod-FieldDefinition
features: [class-static-methods-private, class-static-fields-private, generators, class, class-fields-public]
flags: [generated]
includes: [propertyHelper.js]
info: |
    ClassElement :
      ...
      static FieldDefinition ;

    FieldDefinition :
      ClassElementName Initializer_opt

    ClassElementName :
      PrivateName

    PrivateName :
      # IdentifierName

---*/


var C = class {
  *m() { return 42; } static #xVal; static #yVal;
  static #x(value) {
    this.#xVal = value;
    return this.#xVal;
  }
  static #y(value) {
    this.#yVal = value;
    return this.#yVal;
  }
  static x() {
    return this.#x(42);
  }
  static y() {
    return this.#y(43);
  }
}

var c = new C();

assert.sameValue(c.m().next().value, 42);
assert(
  !Object.prototype.hasOwnProperty.call(c, "m"),
  "m doesn't appear as an own property on the C instance"
);
assert.sameValue(c.m, C.prototype.m);

verifyProperty(C.prototype, "m", {
  enumerable: false,
  configurable: true,
  writable: true,
});

// Test the private methods do not appear as properties before set to value
assert(!Object.prototype.hasOwnProperty.call(C.prototype, "#x"), "test 1");
assert(!Object.prototype.hasOwnProperty.call(C, "#x"), "test 2");
assert(!Object.prototype.hasOwnProperty.call(c, "#x"), "test 3");

assert(!Object.prototype.hasOwnProperty.call(C.prototype, "#y"), "test 4");
assert(!Object.prototype.hasOwnProperty.call(C, "#y"), "test 5");
assert(!Object.prototype.hasOwnProperty.call(c, "#y"), "test 6");

assert(!Object.prototype.hasOwnProperty.call(C.prototype, "#xVal"), "test 7");
assert(!Object.prototype.hasOwnProperty.call(C, "#xVal"), "test 8");
assert(!Object.prototype.hasOwnProperty.call(c, "#xVal"), "test 9");

assert(!Object.prototype.hasOwnProperty.call(C.prototype, "#yVal"), "test 10");
assert(!Object.prototype.hasOwnProperty.call(C, "#yVal"), "test 11");
assert(!Object.prototype.hasOwnProperty.call(c, "#yVal"), "test 12");

// Test if private fields can be sucessfully accessed and set to value
assert.sameValue(C.x(), 42, "test 13");
assert.sameValue(C.y(), 43, "test 14");

// Test the private fields do not appear as properties before after set to value
assert(!Object.prototype.hasOwnProperty.call(C, "#x"), "test 15");
assert(!Object.prototype.hasOwnProperty.call(C, "#y"), "test 16");

assert(!Object.prototype.hasOwnProperty.call(C, "#xVal"), "test 17");
assert(!Object.prototype.hasOwnProperty.call(C, "#yVal"), "test 18");
