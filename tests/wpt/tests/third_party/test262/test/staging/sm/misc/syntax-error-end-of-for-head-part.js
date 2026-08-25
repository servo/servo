/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Syntax errors at the end of |for| statement header parts shouldn't cause crashes
info: bugzilla.mozilla.org/show_bug.cgi?id=672854
esid: pending
---*/

function checkSyntaxError(str)
{
  assert.throws(SyntaxError, function() {
    Function(str);
  });
}

checkSyntaxError("for(var w in \\");
checkSyntaxError("for(w in \\");
checkSyntaxError("for(var w\\");
checkSyntaxError("for(w\\");
checkSyntaxError("for(var w;\\");
checkSyntaxError("for(w;\\");
checkSyntaxError("for(var w; w >\\");
checkSyntaxError("for(w; w >\\");
checkSyntaxError("for(var w; w > 3;\\");
checkSyntaxError("for(w; w > 3;\\");
checkSyntaxError("for(var w; w > 3; 5\\");
checkSyntaxError("for(w; w > 3; 5\\");
checkSyntaxError("for(var w; w > 3; 5foo");
checkSyntaxError("for(w; w > 3; 5foo");
