"use strict";

test(function() {
  assert_equals(self.origin, "http://" + location.host);
}, "self.origin should be correct");
