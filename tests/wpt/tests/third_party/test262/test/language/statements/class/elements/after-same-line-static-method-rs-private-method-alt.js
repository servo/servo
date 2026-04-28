// This file was procedurally generated from the following sources:
// - src/class-elements/rs-private-method-alt.case
// - src/class-elements/productions/cls-decl-after-same-line-static-method.template
/*---
description: Valid PrivateName as private method (field definitions after a static method in the same line)
esid: prod-FieldDefinition
features: [class-methods-private, class-fields-private, class, class-fields-public]
flags: [generated]
includes: [propertyHelper.js]
info: |
    ClassElement :
      MethodDefinition
      ...
      ;

    MethodDefinition :
      ClassElementName ( UniqueFormalParameters ) { FunctionBody }
      ...

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
  static m() { return 42; } #$_; #__; #\u{6F}_; #℘_; #ZW_‌_NJ_; #ZW_‍_J_;
  #$() {
    return this.#$_;
  }
  #_() {
    return this.#__;
  }
  #\u{6F}() {
    return this.#\u{6F}_;
  }
  #℘() {
    return this.#℘_;
  }
  #ZW_‌_NJ() {
    return this.#ZW_‌_NJ_;
  }
  #ZW_‍_J() {
    return this.#ZW_‍_J_;
  }
;
  $(value) {
    this.#$_ = value;
    return this.#$();
  }
  _(value) {
    this.#__ = value;
    return this.#_();
  }
  \u{6F}(value) {
    this.#\u{6F}_ = value;
    return this.#\u{6F}();
  }
  ℘(value) {
    this.#℘_ = value;
    return this.#℘();
  }
  ZW_‌_NJ(value) {
    this.#ZW_‌_NJ_ = value;
    return this.#ZW_‌_NJ();
  }
  ZW_‍_J(value) {
    this.#ZW_‍_J_ = value;
    return this.#ZW_‍_J();
  }

}

var c = new C();

assert.sameValue(C.m(), 42);
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

assert.sameValue(c.$(1), 1);
assert.sameValue(c._(1), 1);
assert.sameValue(c.\u{6F}(1), 1);
assert.sameValue(c.℘(1), 1);
assert.sameValue(c.ZW_‌_NJ(1), 1);
assert.sameValue(c.ZW_‍_J(1), 1);
