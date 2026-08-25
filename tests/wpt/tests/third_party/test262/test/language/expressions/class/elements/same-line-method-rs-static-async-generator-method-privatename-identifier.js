// This file was procedurally generated from the following sources:
// - src/class-elements/rs-static-async-generator-method-privatename-identifier.case
// - src/class-elements/productions/cls-expr-same-line-method.template
/*---
description: Valid Static AsyncGeneratorMethod PrivateName (field definitions followed by a method in the same line)
esid: prod-FieldDefinition
features: [class-static-methods-private, class, class-fields-public]
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
      AsyncGeneratorMethod

    AsyncGeneratorMethod :
      async [no LineTerminator here] * ClassElementName ( UniqueFormalParameters){ AsyncGeneratorBody }

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
  static async * #$(value) {
    yield * await value;
  }
  static async * #_(value) {
    yield * await value;
  }
  static async * #\u{6F}(value) {
    yield * await value;
  }
  static async * #\u2118(value) {
    yield * await value;
  }
  static async * #ZW_\u200C_NJ(value) {
    yield * await value;
  }
  static async * #ZW_\u200D_J(value) {
    yield * await value;
  }; m() { return 42; }
  static get $() {
    return this.#$;
  }
  static get _() {
    return this.#_;
  }
  static get \u{6F}() {
    return this.#\u{6F};
  }
  static get \u2118() {
    return this.#\u2118;
  }
  static get ZW_\u200C_NJ() {
    return this.#ZW_\u200C_NJ;
  }
  static get ZW_\u200D_J() {
    return this.#ZW_\u200D_J;
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

Promise.all([
  C.$([1]).next(),
  C._([1]).next(),
  C.\u{6F}([1]).next(),
  C.\u2118([1]).next(),
  C.ZW_\u200C_NJ([1]).next(),
  C.ZW_\u200D_J([1]).next(),
]).then(results => {

  assert.sameValue(results[0].value, 1);
  assert.sameValue(results[1].value, 1);
  assert.sameValue(results[2].value, 1);
  assert.sameValue(results[3].value, 1);
  assert.sameValue(results[4].value, 1);
  assert.sameValue(results[5].value, 1);

}).then($DONE, $DONE);

