// This file was procedurally generated from the following sources:
// - src/class-elements/rs-static-privatename-identifier-initializer-alt.case
// - src/class-elements/productions/cls-expr-multiple-stacked-definitions.template
/*---
description: Valid Static PrivateName (multiple stacked fields definitions through ASI)
esid: prod-FieldDefinition
features: [class-static-fields-private, class, class-fields-public]
flags: [generated]
includes: [propertyHelper.js]
info: |
    ClassElement :
      MethodDefinition
      static MethodDefinition
      FieldDefinition ;
      static FieldDefinition ;
      ;

    FieldDefinition :
      ClassElementName Initializer _opt

    ClassElementName :
      PropertyName
      PrivateName

    PrivateName ::
      # IdentifierName

    IdentifierName ::
      IdentifierStart
      IdentifierName IdentifierPart

    IdentifierStart ::
      UnicodeIDStart
      $
      _
      \ UnicodeEscapeSequence

    IdentifierPart::
      UnicodeIDContinue
      $
      \ UnicodeEscapeSequence
      <ZWNJ> <ZWJ>

    UnicodeIDStart::
      any Unicode code point with the Unicode property "ID_Start"

    UnicodeIDContinue::
      any Unicode code point with the Unicode property "ID_Continue"


    NOTE 3
    The sets of code points with Unicode properties "ID_Start" and
    "ID_Continue" include, respectively, the code points with Unicode
    properties "Other_ID_Start" and "Other_ID_Continue".

---*/


var C = class {
  static #$ = 1; static #_ = 1; static #\u{6F} = 1; static #℘ = 1; static #ZW_‌_NJ = 1; static #ZW_‍_J = 1
  foo = "foobar"
  bar = "barbaz";
  static $() {
    return this.#$;
  }
  static _() {
    return this.#_;
  }
  static \u{6F}() {
    return this.#\u{6F};
  }
  static ℘() {
    return this.#℘;
  }
  static ZW_‌_NJ() {
    return this.#ZW_‌_NJ;
  }
  static ZW_‍_J() {
    return this.#ZW_‍_J;
  }
}

var c = new C();

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

assert.sameValue(C.$(), 1);
assert.sameValue(C._(), 1);
assert.sameValue(C.\u{6F}(), 1);
assert.sameValue(C.℘(), 1);
assert.sameValue(C.ZW_‌_NJ(), 1);
assert.sameValue(C.ZW_‍_J(), 1);

