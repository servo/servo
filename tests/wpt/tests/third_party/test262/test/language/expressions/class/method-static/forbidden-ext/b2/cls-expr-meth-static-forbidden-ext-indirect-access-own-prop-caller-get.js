// This file was procedurally generated from the following sources:
// - src/function-forms/forbidden-ext-indirect-access-own-prop-caller-get.case
// - src/function-forms/forbidden-extensions/bullet-two/cls-expr-meth-static.template
/*---
description: Forbidden extension, o.caller (static class expression method)
esid: sec-class-definitions-runtime-semantics-evaluation
features: [class]
flags: [generated, noStrict]
info: |
    ClassExpression : class BindingIdentifieropt ClassTail


    If an implementation extends any function object with an own property named "caller" the value of
    that property, as observed using [[Get]] or [[GetOwnProperty]], must not be a strict function
    object. If it is an accessor property, the function that is the value of the property's [[Get]]
    attribute must never return a strict function when called.

---*/
var CALLER_OWN_PROPERTY_DOES_NOT_EXIST = Symbol();
function inner() {
  // This property may exist, but is forbidden from having a value that is a strict function object
  return inner.hasOwnProperty("caller")
    ? inner.caller
    : CALLER_OWN_PROPERTY_DOES_NOT_EXIST;
}

var callCount = 0;
var C = class {
  static method() {
    /* implicit strict */
    // This and the following conditional value is set in the test's .case file.
    // For every test that has a "true" value here, there is a
    // corresponding test that has a "false" value here.
    // They are generated from two different case files, which use
    let descriptor = Object.getOwnPropertyDescriptor(inner, "caller");
    if (descriptor && descriptor.configurable && true) {
      Object.defineProperty(inner, "caller", {get(){return 1}});
    }
    var result = inner();
    if (descriptor && descriptor.configurable && true) {
      assert.sameValue(result, 1, 'If this test defined an own "caller" property on the inner function, then it should be accessible and should return the value it was set to.');
    }

    // "CALLER_OWN_PROPERTY_DOES_NOT_EXIST" is from
    // forbidden-ext-indirect-access-prop-caller.case
    //
    // If the function object "inner" has an own property
    // named "caller", then its value will be returned.
    //
    // If the function object "inner" DOES NOT have an
    // own property named "caller", then the symbol
    // CALLER_OWN_PROPERTY_DOES_NOT_EXIST will be returned.
    if (result !== CALLER_OWN_PROPERTY_DOES_NOT_EXIST) {
      assert.notSameValue(result, this.method);
    }

    callCount++;
  }
};

C.method();
assert.sameValue(callCount, 1, 'method body evaluated');
