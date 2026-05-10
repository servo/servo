// This file was procedurally generated from the following sources:
// - src/class-elements/computed-names.case
// - src/class-elements/productions/cls-expr-after-same-line-async-gen.template
/*---
description: Computed property names (field definitions after an async generator in the same line)
esid: prod-FieldDefinition
features: [class-fields-public, computed-property-names, class, async-iteration]
flags: [generated, async]
includes: [propertyHelper.js]
info: |
    ClassElement:
      ...
      FieldDefinition ;

    FieldDefinition:
      ClassElementName Initializer_opt

    ClassElementName:
      PropertyName

---*/
var x = "b";



var C = class {
  async *m() { return 42; } [x] = 42; [10] = "meep"; ["not initialized"];
  
}

var c = new C();

assert(
  !Object.prototype.hasOwnProperty.call(c, "m"),
  "m doesn't appear as an own property on the C instance"
);
assert.sameValue(c.m, C.prototype.m);

verifyProperty(C.prototype, "m", {
  enumerable: false,
  configurable: true,
  writable: true,
}, {restore: true});

c.m().next().then(function(v) {
  assert.sameValue(v.value, 42);
  assert.sameValue(v.done, true);

  function assertions() {
    // Cover $DONE handler for async cases.
    function $DONE(error) {
      if (error) {
        throw new Test262Error('Test262:AsyncTestFailure')
      }
    }
    assert(
      !Object.prototype.hasOwnProperty.call(C.prototype, "b"),
      "b doesn't appear as an own property on C prototype"
    );
    assert(
      !Object.prototype.hasOwnProperty.call(C, "b"),
      "b doesn't appear as an own property on C constructor"
    );

    verifyProperty(c, "b", {
      value: 42,
      enumerable: true,
      writable: true,
      configurable: true
    });

    assert(
      !Object.prototype.hasOwnProperty.call(C.prototype, "x"),
      "x doesn't appear as an own property on C prototype"
    );
    assert(
      !Object.prototype.hasOwnProperty.call(C, "x"),
      "x doesn't appear as an own property on C constructor"
    );
    assert(
      !Object.prototype.hasOwnProperty.call(c, "x"),
      "x doesn't appear as an own property on C instance"
    );

    assert(
      !Object.prototype.hasOwnProperty.call(C.prototype, "10"),
      "10 doesn't appear as an own property on C prototype"
    );
    assert(
      !Object.prototype.hasOwnProperty.call(C, "10"),
      "10 doesn't appear as an own property on C constructor"
    );

    verifyProperty(c, "10", {
      value: "meep",
      enumerable: true,
      writable: true,
      configurable: true
    });

    assert(
      !Object.prototype.hasOwnProperty.call(C.prototype, "not initialized"),
      "'not initialized' doesn't appear as an own property on C prototype"
    );
    assert(
      !Object.prototype.hasOwnProperty.call(C, "not initialized"),
      "'not initialized' doesn't appear as an own property on C constructor"
    );

    verifyProperty(c, "not initialized", {
      value: undefined,
      enumerable: true,
      writable: true,
      configurable: true
    });
  }

  return Promise.resolve(assertions());
}).then($DONE, $DONE);
