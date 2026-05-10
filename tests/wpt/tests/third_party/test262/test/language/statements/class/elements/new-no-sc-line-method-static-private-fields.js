// This file was procedurally generated from the following sources:
// - src/class-elements/static-private-fields.case
// - src/class-elements/productions/cls-decl-new-no-sc-line-method.template
/*---
description: static private fields (field definitions followed by a method in a new line without a semicolon)
esid: prod-FieldDefinition
features: [class-static-fields-private, class, class-fields-public]
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


class C {
  static #x; static #y
  m() { return 42; }
  static x() {
    this.#x = 42;
    return this.#x;
  }
  static y() {
    this.#y = 43;
    return this.#y;
  }
}

var c = new C();

assert.sameValue(c.m(), 42);
assert.sameValue(c.m, C.prototype.m);
assert(
  !Object.prototype.hasOwnProperty.call(c, "m"),
  "m doesn't appear as an own property on the C instance"
);

verifyProperty(C.prototype, "m", {
  enumerable: false,
  configurable: true,
  writable: true,
});

// Test the private fields do not appear as properties before set to value
assert(!Object.prototype.hasOwnProperty.call(C.prototype, "#x"), "test 1");
assert(!Object.prototype.hasOwnProperty.call(C, "#x"), "test 2");
assert(!Object.prototype.hasOwnProperty.call(c, "#x"), "test 3");

assert(!Object.prototype.hasOwnProperty.call(C.prototype, "#y"), "test 4");
assert(!Object.prototype.hasOwnProperty.call(C, "#y"), "test 5");
assert(!Object.prototype.hasOwnProperty.call(c, "#y"), "test 6");

// Test if private fields can be sucessfully accessed and set to value
assert.sameValue(C.x(), 42, "test 7");
assert.sameValue(C.y(), 43, "test 8");

// Test the private fields do not appear as properties before after set to value
assert(!Object.prototype.hasOwnProperty.call(C, "#x"), "test 9");
assert(!Object.prototype.hasOwnProperty.call(C, "#y"), "test 10");
