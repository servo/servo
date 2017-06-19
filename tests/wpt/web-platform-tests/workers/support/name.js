"use strict";
importScripts("/resources/testharness.js");

test(() => {
  assert_true(self.hasOwnProperty("name"), "property exists on the global")
  assert_equals(self.name, "my name")
}, `name property value for ${self.constructor.name}`);

done();
