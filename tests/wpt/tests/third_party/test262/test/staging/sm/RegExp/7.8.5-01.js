/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Line terminator after backslash is invalid in regexp literals
info: bugzilla.mozilla.org/show_bug.cgi?id=615070
esid: pending
---*/

var regexps = ["/\\\u000A/", "/\\\u000D/", "/\\\u2028/", "/\\\u2029/",
	       "/ab\\\n/", "/ab\\\r/", "/ab\\\u2028/", "/ab\\\u2029/",
	       "/ab[c\\\n]/", "/a[bc\\", "/\\"];

for(var i=0; i<regexps.length; i++) {
    var src = regexps[i];
    assert.throws(SyntaxError, function() {
        eval(src).source;
    });
}
