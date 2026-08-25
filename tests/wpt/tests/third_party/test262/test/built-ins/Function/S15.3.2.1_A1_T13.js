// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When the Function constructor is called with one argument then body be that argument and the following steps are taken:
    i) Call ToString(body)
    ii) If P is not parsable as a FormalParameterListopt then throw a SyntaxError exception
    iii) If body is not parsable as FunctionBody then throw a SyntaxError exception
    iv) Create a new Function object as specified in 13.2 with parameters specified by parsing P as a FormalParameterListopt and body specified by parsing body as a FunctionBody.
    Pass in a scope chain consisting of the global object as the Scope parameter
    v) Return Result(iv)
es5id: 15.3.2.1_A1_T13
description: Value of the function constructor argument is "{}"
---*/

try {
  var f = new Function({});
  throw new Test262Error('#1: test failed with error ' + e);
} catch (e) {
  assert(
    e instanceof SyntaxError,
    'The result of evaluating (e instanceof SyntaxError) is expected to be true'
  );
}
