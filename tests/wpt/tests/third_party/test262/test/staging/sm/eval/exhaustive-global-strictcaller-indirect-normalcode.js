/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - onlyStrict
description: |
  eval in all its myriad flavors
info: bugzilla.mozilla.org/show_bug.cgi?id=514568
esid: pending
---*/

var x = 17;

var ev = eval;

var xcode =
  "var x = 4;" +
  "function actX(action)" +
  "{" +
  "  switch (action)" +
  "  {" +
  "    case 'get':" +
  "      return x;" +
  "    case 'set1':" +
  "      x = 9;" +
  "      return;" +
  "    case 'set2':" +
  "      x = 23;" +
  "      return;" +
  "    case 'delete':" +
  "      try { return eval('delete x'); }" +
  "      catch (e) { return e.name; }" +
  "  }" +
  "}" +
  "actX;";

var local0 = x;

var f = ev(xcode);

var inner1 = f("get");
var local1 = x;

x = 7;
var inner2 = f("get");
var local2 = x;

f("set1");
var inner3 = f("get");
var local3 = x;

var del = f("delete");
var inner4 = f("get");
var local4 = x;

f("set2");
var inner5 = f("get");
var local5 = x;

var resultsX =
  {
     local0: local0,
     inner1: inner1, local1: local1,
     inner2: inner2, local2: local2,
     inner3: inner3, local3: local3,
     del: del,
     inner4: inner4, local4: local4,
     inner5: inner5, local5: local5,
   };

assert.sameValue(resultsX.local0, 17);

assert.sameValue(resultsX.inner1, 4);
assert.sameValue(resultsX.local1, 4);

assert.sameValue(resultsX.inner2, 7);
assert.sameValue(resultsX.local2, 7);

assert.sameValue(resultsX.inner3, 9);
assert.sameValue(resultsX.local3, 9);

assert.sameValue(resultsX.del, false);

assert.sameValue(resultsX.inner4, 9);
assert.sameValue(resultsX.local4, 9);

assert.sameValue(resultsX.inner5, 23);
assert.sameValue(resultsX.local5, 23);


var ycode =
  "var y = 5;" +
  "function actY(action)" +
  "{" +
  "  switch (action)" +
  "  {" +
  "    case 'get':" +
  "      return y;" +
  "    case 'set1':" +
  "      y = 2;" +
  "      return;" +
  "    case 'set2':" +
  "      y = 71;" +
  "      return;" +
  "    case 'delete':" +
  "      try { return eval('delete y'); }" +
  "      catch (e) { return e.name; }" +
  "  }" +
  "}" +
  "actY;";

try { var local0 = y; } catch (e) { local0 = e.name; }

var f = ev(ycode);

var inner1 = f("get");
var local1 = y;

try { y = 8; } catch (e) { assert.sameValue(e.name, "ReferenceError"); }
var inner2 = f("get");
var local2 = y;

f("set1");
var inner3 = f("get");
var local3 = y;

var del = f("delete");
try { var inner4 = f("get"); } catch (e) { inner4 = e.name; }
try { var local4 = y; } catch (e) { local4 = e.name; }

f("set2");
try { var inner5 = f("get"); } catch (e) { inner5 = e.name; }
try { var local5 = y; } catch (e) { local5 = e.name; }

var resultsY =
  {
    local0: local0,
    inner1: inner1, local1: local1,
    inner2: inner2, local2: local2,
    inner3: inner3, local3: local3,
    del: del,
    inner4: inner4, local4: local4,
    inner5: inner5, local5: local5,
  };

assert.sameValue(resultsY.local0, "ReferenceError");

assert.sameValue(resultsY.inner1, 5);
assert.sameValue(resultsY.local1, 5);

assert.sameValue(resultsY.inner2, 8);
assert.sameValue(resultsY.local2, 8);

assert.sameValue(resultsY.inner3, 2);
assert.sameValue(resultsY.local3, 2);

assert.sameValue(resultsY.del, true);

assert.sameValue(resultsY.inner4, "ReferenceError");
assert.sameValue(resultsY.local4, "ReferenceError");

assert.sameValue(resultsY.inner5, 71);
assert.sameValue(resultsY.local5, 71);
