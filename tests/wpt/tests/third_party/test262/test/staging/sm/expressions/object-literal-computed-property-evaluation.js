/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Convert computed property name expressions to property key before evaluating the property's value
info: bugzilla.mozilla.org/show_bug.cgi?id=1199546
esid: pending
---*/

var s = "foo";
var convertsToS = { toString() { return s; } };

var o = {
  [convertsToS]: // after ToPropertyKey becomes "foo"
    (function() {
      s = 'bar';
      return 'abc'; // so we have "foo": "bar" for the first property
     })(),

  [convertsToS]: // |s| was set above to "bar", so after ToPropertyKey, "bar"
    'efg' // so we have "bar": "efg" for the second property
};

assert.sameValue(o.foo, "abc");
assert.sameValue(o.bar, "efg");
