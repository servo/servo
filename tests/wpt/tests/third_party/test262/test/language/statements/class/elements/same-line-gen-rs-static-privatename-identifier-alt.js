// This file was procedurally generated from the following sources:
// - src/class-elements/rs-static-privatename-identifier-alt.case
// - src/class-elements/productions/cls-decl-same-line-generator.template
/*---
description: Valid Static PrivateName (field definitions followed by a generator method in the same line)
esid: prod-FieldDefinition
features: [class-static-fields-private, class, class-fields-public, generators]
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


class C {
  static #$; static #_; static #\u{6F}; static #℘; static #ZW_‌_NJ; static #ZW_‍_J; *m() { return 42; }
  static $(value) {
    this.#$ = value;
    return this.#$;
  }
  static _(value) {
    this.#_ = value;
    return this.#_;
  }
  static o(value) {
    this.#\u{6F} = value;
    return this.#\u{6F};
  }
  static ℘(value) { // DO NOT CHANGE THE NAME OF THIS FIELD
    this.#℘ = value;
    return this.#℘;
  }
  static ZW_‌_NJ(value) { // DO NOT CHANGE THE NAME OF THIS FIELD
    this.#ZW_‌_NJ = value;
    return this.#ZW_‌_NJ;
  }
  static ZW_‍_J(value) { // DO NOT CHANGE THE NAME OF THIS FIELD
    this.#ZW_‍_J = value;
    return this.#ZW_‍_J;
  }
}

var c = new C();

assert.sameValue(c.m().next().value, 42);
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

assert.sameValue(C.$(1), 1);
assert.sameValue(C._(1), 1);
assert.sameValue(C.o(1), 1);
assert.sameValue(C.℘(1), 1);      // DO NOT CHANGE THE NAME OF THIS FIELD
assert.sameValue(C.ZW_‌_NJ(1), 1); // DO NOT CHANGE THE NAME OF THIS FIELD
assert.sameValue(C.ZW_‍_J(1), 1);  // DO NOT CHANGE THE NAME OF THIS FIELD

