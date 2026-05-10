// This file was procedurally generated from the following sources:
// - src/class-elements/string-literal-names.case
// - src/class-elements/productions/cls-expr-multiple-definitions.template
/*---
description: String literal names (multiple fields definitions)
esid: prod-FieldDefinition
features: [class-fields-public, class]
flags: [generated]
includes: [propertyHelper.js]
info: |
    ClassElement:
      ...
      FieldDefinition ;

    FieldDefinition:
      ClassElementName Initializer_opt

    ClassElementName:
      PropertyName

---*/


var C = class {
  foo = "foobar";
  m() { return 42 }
  'a'; "b"; 'c' = 39;
  "d" = 42
  m2() { return 39 }
  bar = "barbaz";
  
}

var c = new C();

assert.sameValue(c.m(), 42);
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

assert.sameValue(c.m2(), 39);
assert(
  !Object.prototype.hasOwnProperty.call(c, "m2"),
  "m2 doesn't appear as an own property on the C instance"
);
assert.sameValue(c.m2, C.prototype.m2);

verifyProperty(C.prototype, "m2", {
  enumerable: false,
  configurable: true,
  writable: true,
});

assert.sameValue(c.foo, "foobar");
assert(
  !Object.prototype.hasOwnProperty.call(C, "foo"),
  "foo doesn't appear as an own property on the C constructor"
);
assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, "foo"),
  "foo doesn't appear as an own property on the C prototype"
);

verifyProperty(c, "foo", {
  value: "foobar",
  enumerable: true,
  configurable: true,
  writable: true,
});

assert.sameValue(c.bar, "barbaz");
assert(
  !Object.prototype.hasOwnProperty.call(C, "bar"),
  "bar doesn't appear as an own property on the C constructor"
);
assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, "bar"),
  "bar doesn't appear as an own property on the C prototype"
);

verifyProperty(c, "bar", {
  value: "barbaz",
  enumerable: true,
  configurable: true,
  writable: true,
});

assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, "a"),
  "a does not appear as an own property on C prototype"
);
assert(
  !Object.prototype.hasOwnProperty.call(C, "a"),
  "a does not appear as an own property on C constructor"
);

verifyProperty(c, "a", {
  value: undefined,
  enumerable: true,
  writable: true,
  configurable: true
});

assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, "b"),
  "b does not appear as an own property on C prototype"
);
assert(
  !Object.prototype.hasOwnProperty.call(C, "b"),
  "b does not appear as an own property on C constructor"
);

verifyProperty(c, "b", {
  value: undefined,
  enumerable: true,
  writable: true,
  configurable: true
});

assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, "c"),
  "c does not appear as an own property on C prototype"
);
assert(
  !Object.prototype.hasOwnProperty.call(C, "c"),
  "c does not appear as an own property on C constructor"
);

verifyProperty(c, "c", {
  value: 39,
  enumerable: true,
  writable: true,
  configurable: true
});

assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, "d"),
  "d does not appear as an own property on C prototype"
);
assert(
  !Object.prototype.hasOwnProperty.call(C, "d"),
  "d does not appear as an own property on C constructor"
);

verifyProperty(c, "d", {
  value: 42,
  enumerable: true,
  writable: true,
  configurable: true
});
