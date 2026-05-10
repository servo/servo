/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/

function throwsNoSyntaxError(code) {
  eval(code);
};

function throwsSyntaxError(code) {
  assert.throws(SyntaxError, function() {
    eval(code);
  });
};

/* 
 * Duplicate parameter names must be tolerated (as per ES3), unless
 * the parameter list uses destructuring, in which case we claim the
 * user has opted in to a modicum of sanity, and we forbid duplicate
 * parameter names.
 */
throwsNoSyntaxError("function f(x,x){}");

throwsSyntaxError("function f(x,[x]){})");
throwsSyntaxError("function f(x,{y:x}){})");
throwsSyntaxError("function f(x,{x}){})");

throwsSyntaxError("function f([x],x){})");
throwsSyntaxError("function f({y:x},x){})");
throwsSyntaxError("function f({x},x){})");

throwsSyntaxError("function f([x,x]){}");
throwsSyntaxError("function f({x,x}){}");
throwsSyntaxError("function f({y:x,z:x}){}");

throwsSyntaxError("function f(x,x,[y]){}");
throwsSyntaxError("function f(x,x,{y}){}");
throwsSyntaxError("function f([y],x,x){}");
throwsSyntaxError("function f({y},x,x){}");

throwsSyntaxError("function f(a,b,c,d,e,f,g,h,b,[y]){}");
throwsSyntaxError("function f([y],a,b,c,d,e,f,g,h,a){}");
throwsSyntaxError("function f([a],b,c,d,e,f,g,h,i,a){}");
throwsSyntaxError("function f(a,b,c,d,e,f,g,h,i,[a]){}");
throwsSyntaxError("function f(a,b,c,d,e,f,g,h,i,[a]){}");
