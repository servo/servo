// META: title=The self attribute
// META: global=shadowrealm

test(() => {
  assert_equals(self, globalThis, "self should be the same object as globalThis");
}, "self attribute is the global object");

test(() => {
  assert_equals(self, self.self, "self should be the same object as self.self");
}, "self attribute is the object itself");

test(() => {
  assert_own_property(globalThis, "self", "self should be an own property");
  assert_readonly(globalThis, "self", "self should be a read-only property");
}, "self is a readonly attribute");

test(() => {
  // https://webidl.spec.whatwg.org/#define-the-attributes
  const descr = Object.getOwnPropertyDescriptor(self, "self");
  assert_equals(descr.value, undefined, "self should be an accessor property");
  assert_true(descr.enumerable, "self should be enumerable");
  assert_true(descr.configurable, "self should be configurable");
}, "self property descriptor");

test(() => {
  const getter = Object.getOwnPropertyDescriptor(self, "self").get;
  assert_equals(getter.name, "get self", "function should be named 'get self'");
}, "self getter name");

test(() => {
  const getter = Object.getOwnPropertyDescriptor(self, "self").get;
  assert_equals(getter.length, 0, "function should take 0 arguments");
}, "self getter length");

test(() => {
  // https://webidl.spec.whatwg.org/#dfn-attribute-getter
  const getter = Object.getOwnPropertyDescriptor(self, "self").get;

  assert_throws_js(TypeError, () => getter.call({}),
    "the self getter should fail a brand check if it's an object not implementing ShadowRealmGlobalScope");
  assert_throws_js(TypeError, () => getter.call(42),
    "the self getter should fail a brand check if a primitive");

  assert_equals(getter.call(null), self,
    "the self getter's this object should fall back to the realm's global object if null");
  assert_equals(getter.call(undefined), self,
    "the self getter's this object should fall back to the realm's global object if undefined");
}, "self getter steps");
