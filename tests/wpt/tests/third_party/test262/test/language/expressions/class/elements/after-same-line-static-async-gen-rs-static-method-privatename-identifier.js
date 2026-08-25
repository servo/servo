// This file was procedurally generated from the following sources:
// - src/class-elements/rs-static-method-privatename-identifier.case
// - src/class-elements/productions/cls-expr-after-same-line-static-async-gen.template
/*---
description: Valid Static Method PrivateName (field definitions after a static async generator in the same line)
esid: prod-FieldDefinition
features: [class-static-methods-private, class, class-fields-public, async-iteration]
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
      ClassElementName ( UniqueFormalParameters ){ FunctionBody }

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
  static async *m() { return 42; } static #$(value) {
    return value;
  }
  static #_(value) {
    return value;
  }
  static #\u{6F}(value) {
    return value;
  }
  static #\u2118(value) {
    return value;
  }
  static #ZW_\u200C_NJ(value) {
    return value;
  }
  static #ZW_\u200D_J(value) {
    return value;
  };
  static $(value) {
    return this.#$(value);
  }
  static _(value) {
    return this.#_(value);
  }
  static \u{6F}(value) {
    return this.#\u{6F}(value);
  }
  static \u2118(value) {
    return this.#\u2118(value);
  }
  static ZW_\u200C_NJ(value) {
    return this.#ZW_\u200C_NJ(value);
  }
  static ZW_\u200D_J(value) {
    return this.#ZW_\u200D_J(value);
  }

}

var c = new C();

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
}, {restore: true});

C.m().next().then(function(v) {
  assert.sameValue(v.value, 42);
  assert.sameValue(v.done, true);

  function assertions() {
    // Cover $DONE handler for async cases.
    function $DONE(error) {
      if (error) {
        throw new Test262Error('Test262:AsyncTestFailure')
      }
    }
    assert.sameValue(C.$(1), 1);
    assert.sameValue(C._(1), 1);
    assert.sameValue(C.\u{6F}(1), 1);
    assert.sameValue(C.\u2118(1), 1);
    assert.sameValue(C.ZW_\u200C_NJ(1), 1);
    assert.sameValue(C.ZW_\u200D_J(1), 1);

  }

  return Promise.resolve(assertions());
}).then($DONE, $DONE);
