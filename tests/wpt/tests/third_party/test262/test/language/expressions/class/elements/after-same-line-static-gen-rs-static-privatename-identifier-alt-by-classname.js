// This file was procedurally generated from the following sources:
// - src/class-elements/rs-static-privatename-identifier-alt-by-classname.case
// - src/class-elements/productions/cls-expr-after-same-line-static-gen.template
/*---
description: Valid Static PrivateName (field definitions after a static generator in the same line)
esid: prod-FieldDefinition
features: [class-static-fields-private, generators, class, class-fields-public]
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
  static *m() { return 42; } static #$; static #_; static #\u{6F}; static #℘; static #ZW_‌_NJ; static #ZW_‍_J;
  static $(value) {
    C.#$ = value;
    return C.#$;
  }
  static _(value) {
    C.#_ = value;
    return C.#_;
  }
  static o(value) {
    C.#\u{6F} = value;
    return C.#\u{6F};
  }
  static ℘(value) { // DO NOT CHANGE THE NAME OF THIS FIELD
    C.#℘ = value;
    return C.#℘;
  }
  static ZW_‌_NJ(value) { // DO NOT CHANGE THE NAME OF THIS FIELD
    C.#ZW_‌_NJ = value;
    return C.#ZW_‌_NJ;
  }
  static ZW_‍_J(value) { // DO NOT CHANGE THE NAME OF THIS FIELD
    C.#ZW_‍_J = value;
    return C.#ZW_‍_J;
  }
}

var c = new C();

assert.sameValue(C.m().next().value, 42);
assert(
  !Object.prototype.hasOwnProperty.call(c, "m"),
  "m doesn't appear as an own property on the C instance"
);
assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, "m"),
  "m doesn't appear as an own property on the C prototype"
);

verifyProperty(C, "m", {
  enumerable: false,
  configurable: true,
  writable: true,
});

assert.sameValue(C.$(1), 1);
assert.sameValue(C._(1), 1);
assert.sameValue(C.o(1), 1);
assert.sameValue(C.℘(1), 1);      // DO NOT CHANGE THE NAME OF THIS FIELD
assert.sameValue(C.ZW_‌_NJ(1), 1); // DO NOT CHANGE THE NAME OF THIS FIELD
assert.sameValue(C.ZW_‍_J(1), 1);  // DO NOT CHANGE THE NAME OF THIS FIELD

