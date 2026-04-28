// This file was procedurally generated from the following sources:
// - src/class-elements/rs-static-async-method-privatename-identifier.case
// - src/class-elements/productions/cls-expr-after-same-line-static-gen.template
/*---
description: Valid Static AsyncMethod PrivateName (field definitions after a static generator in the same line)
esid: prod-FieldDefinition
features: [class-static-methods-private, generators, class, class-fields-public]
flags: [generated, async]
includes: [propertyHelper.js]
info: |
    ClassElement :
      MethodDefinition
      static MethodDefinition
      FieldDefinition ;
      static FieldDefinition ;
      ;

    MethodDefinition :
      AsyncMethod

    AsyncMethod :
      async [no LineTerminator here] ClassElementName ( UniqueFormalParameters ){ AsyncFunctionBody }

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
  static *m() { return 42; } static async #$(value) {
    return await value;
  }
  static async #_(value) {
    return await value;
  }
  static async #\u{6F}(value) {
    return await value;
  }
  static async #\u2118(value) {
    return await value;
  }
  static async #ZW_\u200C_NJ(value) {
    return await value;
  }
  static async #ZW_\u200D_J(value) {
    return await value;
  };
  static async $(value) {
    return await this.#$(value);
  }
  static async _(value) {
    return await this.#_(value);
  }
  static async \u{6F}(value) {
    return await this.#\u{6F}(value);
  }
  static async \u2118(value) {
    return await this.#\u2118(value);
  }
  static async ZW_\u200C_NJ(value) {
    return await this.#ZW_\u200C_NJ(value);
  }
  static async ZW_\u200D_J(value) {
    return await this.#ZW_\u200D_J(value);
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

Promise.all([
  C.$(1),
  C._(1),
  C.\u{6F}(1),
  C.\u2118(1),
  C.ZW_\u200C_NJ(1),
  C.ZW_\u200D_J(1),
]).then(results => {

  assert.sameValue(results[0], 1);
  assert.sameValue(results[1], 1);
  assert.sameValue(results[2], 1);
  assert.sameValue(results[3], 1);
  assert.sameValue(results[4], 1);
  assert.sameValue(results[5], 1);

}).then($DONE, $DONE);

