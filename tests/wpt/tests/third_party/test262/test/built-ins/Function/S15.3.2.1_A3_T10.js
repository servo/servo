// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When the Function constructor is called with arguments p, body the following steps are taken:
    i) Let Result(i) be the first argument
    ii) Let P be ToString(Result(i))
    iii) Call ToString(body)
    iv) If P is not parsable as a FormalParameterList_opt then throw a SyntaxError exception
    v) If body is not parsable as FunctionBody then throw a SyntaxError exception
    vi) Create a new Function object as specified in 13.2 with parameters specified by parsing P as a FormalParameterListopt and body specified by parsing body as a FunctionBody
    Pass in a scope chain consisting of the global object as the Scope parameter
    vii) Return Result(vi)
es5id: 15.3.2.1_A3_T10
description: >
    Values of the function constructor arguments are
    "{toString:function(){return "z;x"}}" and "return this;"
---*/

var body = "return this;";
var p = {
  toString: function() {
    return "z;x"
  }
};

try {
  var f = new Function(p, body);
  throw new Test262Error('#1: If P is not parsable as a FormalParameterList_opt then throw a SyntaxError exception');
} catch (e) {
  assert(
    e instanceof SyntaxError,
    'The result of evaluating (e instanceof SyntaxError) is expected to be true'
  );
}
