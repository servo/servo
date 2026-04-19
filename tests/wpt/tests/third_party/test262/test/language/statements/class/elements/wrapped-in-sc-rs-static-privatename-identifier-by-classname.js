// This file was procedurally generated from the following sources:
// - src/class-elements/rs-static-privatename-identifier-by-classname.case
// - src/class-elements/productions/cls-decl-wrapped-in-sc.template
/*---
description: Valid Static PrivateName (fields definition wrapped in semicolons)
esid: prod-FieldDefinition
features: [class-static-fields-private, class, class-fields-public]
flags: [generated]
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
  ;;;;
  ;;;;;;static #$; static #_; static #\u{6F}; static #\u2118; static #ZW_\u200C_NJ; static #ZW_\u200D_J;;;;;;;
  ;;;;
  static $(value) {
    C.#$ = value;
    return C.#$;
  }
  static _(value) {
    C.#_ = value;
    return C.#_;
  }
  static \u{6F}(value) {
    C.#\u{6F} = value;
    return C.#\u{6F};
  }
  static \u2118(value) {
    C.#\u2118 = value;
    return C.#\u2118;
  }
  static ZW_\u200C_NJ(value) {
    C.#ZW_\u200C_NJ = value;
    return C.#ZW_\u200C_NJ;
  }
  static ZW_\u200D_J(value) {
    C.#ZW_\u200D_J = value;
    return C.#ZW_\u200D_J;
  }
}

var c = new C();

assert.sameValue(C.$(1), 1);
assert.sameValue(C._(1), 1);
assert.sameValue(C.\u{6F}(1), 1);
assert.sameValue(C.\u2118(1), 1);
assert.sameValue(C.ZW_\u200C_NJ(1), 1);
assert.sameValue(C.ZW_\u200D_J(1), 1);

