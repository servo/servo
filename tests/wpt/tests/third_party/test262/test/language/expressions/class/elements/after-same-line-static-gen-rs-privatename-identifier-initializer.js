// This file was procedurally generated from the following sources:
// - src/class-elements/rs-privatename-identifier-initializer.case
// - src/class-elements/productions/cls-expr-after-same-line-static-gen.template
/*---
description: Valid PrivateName (field definitions after a static generator in the same line)
esid: prod-FieldDefinition
features: [class-fields-private, generators, class, class-fields-public]
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
  static *m() { return 42; } #$ = 1; #_ = 1; #\u{6F} = 1; #\u2118 = 1; #ZW_\u200C_NJ = 1; #ZW_\u200D_J = 1;
  $() {
    return this.#$;
  }
  _() {
    return this.#_;
  }
  \u{6F}() {
    return this.#\u{6F};
  }
  \u2118() {
    return this.#\u2118;
  }
  ZW_\u200C_NJ() {
    return this.#ZW_\u200C_NJ;
  }
  ZW_\u200D_J() {
    return this.#ZW_\u200D_J;
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

assert.sameValue(c.$(), 1);
assert.sameValue(c._(), 1);
assert.sameValue(c.\u{6F}(), 1);
assert.sameValue(c.\u2118(), 1);
assert.sameValue(c.ZW_\u200C_NJ(), 1);
assert.sameValue(c.ZW_\u200D_J(), 1);

