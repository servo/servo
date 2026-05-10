// This file was procedurally generated from the following sources:
// - src/class-elements/rs-privatename-identifier-alt.case
// - src/class-elements/productions/cls-expr-new-sc-line-generator.template
/*---
description: Valid PrivateName (field definitions followed by a method in a new line with a semicolon)
esid: prod-FieldDefinition
features: [class-fields-private, class, class-fields-public, generators]
flags: [generated]
includes: [propertyHelper.js]
info: |
    ClassElement :
      MethodDefinition
      static MethodDefinition
      FieldDefinition ;
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
  #$; #_; #\u{6F}; #℘; #ZW_‌_NJ; #ZW_‍_J;
  *m() { return 42; }
  $(value) {
    this.#$ = value;
    return this.#$;
  }
  _(value) {
    this.#_ = value;
    return this.#_;
  }
  \u{6F}(value) {
    this.#\u{6F} = value;
    return this.#\u{6F};
  }
  ℘(value) {
    this.#℘ = value;
    return this.#℘;
  }
  ZW_‌_NJ(value) {
    this.#ZW_‌_NJ = value;
    return this.#ZW_‌_NJ;
  }
  ZW_‍_J(value) {
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

assert.sameValue(c.$(1), 1);
assert.sameValue(c._(1), 1);
assert.sameValue(c.\u{6F}(1), 1);
assert.sameValue(c.℘(1), 1);
assert.sameValue(c.ZW_‌_NJ(1), 1);
assert.sameValue(c.ZW_‍_J(1), 1);

