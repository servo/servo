"use strict";
self.importScripts('/resources/testharness.js');

test(() => {
  assert_equals(self.applicationCache, undefined, "self.applicationCache must be undefined");
  assert_false("applicationCache" in self, "applicationCache must not even be a property of self");
}, "self.applicationCache must not exist");

test(() => {
  assert_equals(self.ApplicationCache, undefined, "self.ApplicationCache must be undefined");
  assert_false("ApplicationCache" in self, "ApplicationCache must not even be a property of self");
}, "ApplicationCache must not be exposed");

done();
