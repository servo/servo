/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Test for correct short-circuiting implementation of Date.set methods
esid: pending
---*/

var global = 0;
var date;

/* Test that methods don't short circuit argument evaluation. */
date = new Date(0).setSeconds(NaN, {valueOf:function(){global = 3}});
assert.sameValue(global, 3);

date = new Date(0).setUTCSeconds(NaN, {valueOf:function(){global = 4}});
assert.sameValue(global, 4);

date = new Date(0).setMinutes(NaN, {valueOf:function(){global = 5}});
assert.sameValue(global, 5);

date = new Date(0).setUTCMinutes(NaN, {valueOf:function(){global = 6}});
assert.sameValue(global, 6);

date = new Date(0).setHours(NaN, {valueOf:function(){global = 7}});
assert.sameValue(global, 7);

date = new Date(0).setUTCHours(NaN, {valueOf:function(){global = 8}});
assert.sameValue(global, 8);

date = new Date(0).setMonth(NaN, {valueOf:function(){global = 11}});
assert.sameValue(global, 11);

date = new Date(0).setUTCMonth(NaN, {valueOf:function(){global = 12}});
assert.sameValue(global, 12);

date = new Date(0).setFullYear(NaN, {valueOf:function(){global = 13}});
assert.sameValue(global, 13);

date = new Date(0).setUTCFullYear(NaN, {valueOf:function(){global = 14}});
assert.sameValue(global, 14);



/* Test that argument evaluation is not short circuited if Date == NaN */
date = new Date(NaN).setMilliseconds({valueOf:function(){global = 15}});
assert.sameValue(global, 15);

date = new Date(NaN).setUTCMilliseconds({valueOf:function(){global = 16}});
assert.sameValue(global, 16);

date = new Date(NaN).setSeconds({valueOf:function(){global = 17}});
assert.sameValue(global, 17);

date = new Date(NaN).setUTCSeconds({valueOf:function(){global = 18}});
assert.sameValue(global, 18);

date = new Date(NaN).setMinutes({valueOf:function(){global = 19}});
assert.sameValue(global, 19);

date = new Date(NaN).setUTCMinutes({valueOf:function(){global = 20}});
assert.sameValue(global, 20);

date = new Date(NaN).setHours({valueOf:function(){global = 21}});
assert.sameValue(global, 21);

date = new Date(NaN).setUTCHours({valueOf:function(){global = 22}});
assert.sameValue(global, 22);

date = new Date(NaN).setDate({valueOf:function(){global = 23}});
assert.sameValue(global, 23);

date = new Date(NaN).setUTCDate({valueOf:function(){global = 24}});
assert.sameValue(global, 24);

date = new Date(NaN).setMonth({valueOf:function(){global = 25}});
assert.sameValue(global, 25);

date = new Date(NaN).setUTCMonth({valueOf:function(){global = 26}});
assert.sameValue(global, 26);

date = new Date(NaN).setFullYear({valueOf:function(){global = 27}});
assert.sameValue(global, 27);

date = new Date(NaN).setUTCFullYear({valueOf:function(){global = 28}});
assert.sameValue(global, 28);


/* Test the combination of the above two. */
date = new Date(NaN).setSeconds(NaN, {valueOf:function(){global = 31}});
assert.sameValue(global, 31);

date = new Date(NaN).setUTCSeconds(NaN, {valueOf:function(){global = 32}});
assert.sameValue(global, 32);

date = new Date(NaN).setMinutes(NaN, {valueOf:function(){global = 33}});
assert.sameValue(global, 33);

date = new Date(NaN).setUTCMinutes(NaN, {valueOf:function(){global = 34}});
assert.sameValue(global, 34);

date = new Date(NaN).setHours(NaN, {valueOf:function(){global = 35}});
assert.sameValue(global, 35);

date = new Date(NaN).setUTCHours(NaN, {valueOf:function(){global = 36}});
assert.sameValue(global, 36);

date = new Date(NaN).setMonth(NaN, {valueOf:function(){global = 39}});
assert.sameValue(global, 39);

date = new Date(NaN).setUTCMonth(NaN, {valueOf:function(){global = 40}});
assert.sameValue(global, 40);

date = new Date(NaN).setFullYear(NaN, {valueOf:function(){global = 41}});
assert.sameValue(global, 41);

date = new Date(NaN).setUTCFullYear(NaN, {valueOf:function(){global = 42}});
assert.sameValue(global, 42);


/*Test two methods evaluation*/
var secondGlobal = 0;

date = new Date(NaN).setFullYear({valueOf:function(){global = 43}}, {valueOf:function(){secondGlobal = 1}});
assert.sameValue(global, 43);
assert.sameValue(secondGlobal, 1);

date = new Date(0).setFullYear(NaN, {valueOf:function(){global = 44}}, {valueOf:function(){secondGlobal = 2}});
assert.sameValue(global, 44);
assert.sameValue(secondGlobal, 2);


/* Test year methods*/
date = new Date(0).setYear({valueOf:function(){global = 45}});
assert.sameValue(global, 45);

date = new Date(NaN).setYear({valueOf:function(){global = 46}});
assert.sameValue(global, 46);
