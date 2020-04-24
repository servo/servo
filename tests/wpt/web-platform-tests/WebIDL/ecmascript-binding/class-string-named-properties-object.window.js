"use strict";

const namedPropertiesObject = Object.getPrototypeOf(Window.prototype);

test(() => {
  assert_own_property(namedPropertiesObject, Symbol.toStringTag);

  const propDesc = Object.getOwnPropertyDescriptor(namedPropertiesObject, Symbol.toStringTag);
  assert_equals(propDesc.value, "WindowProperties", "value");
  assert_equals(propDesc.configurable, true, "configurable");
  assert_equals(propDesc.enumerable, false, "enumerable");
  assert_equals(propDesc.writable, false, "writable");
}, "@@toStringTag exists with the appropriate descriptor");

test(() => {
  assert_equals(Object.prototype.toString.call(namedPropertiesObject), "[object WindowProperties]");
}, "Object.prototype.toString");

test(t => {
  assert_own_property(namedPropertiesObject, Symbol.toStringTag, "Precondition for this test: @@toStringTag exists");

  t.add_cleanup(() => {
    Object.defineProperty(namedPropertiesObject, Symbol.toStringTag, { value: "WindowProperties" });
  });

  Object.defineProperty(namedPropertiesObject, Symbol.toStringTag, { value: "NotWindowProperties" });
  assert_equals(Object.prototype.toString.call(namedPropertiesObject), "[object NotWindowProperties]");
}, "Object.prototype.toString applied after modifying @@toStringTag");

// Chrome had a bug (https://bugs.chromium.org/p/chromium/issues/detail?id=793406) where if there
// was no @@toStringTag, it would fall back to a magic class string. This tests that the bug is
// fixed.

// Note: we cannot null out the prototype of the named properties object per
// https://heycam.github.io/webidl/#named-properties-object-setprototypeof so we don't have a test that does that.

// This test must be last.
test(() => {
  delete namedPropertiesObject[Symbol.toStringTag];

  assert_equals(Object.prototype.toString.call(namedPropertiesObject), "[object EventTarget]", "prototype");
}, "Object.prototype.toString applied after deleting @@toStringTag");
