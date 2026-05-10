/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/

/*
 * For the sake of cross compatibility with other implementations we
 * follow the W3C "NOTE-datetime" specification when parsing dates of
 * the form YYYY-MM-DDTHH:MM:SS save for a few exceptions: months, days, hours
 * minutes, and seconds may be either one _or_ two digits long, and the 'T'
 * preceding the time part may be replaced with a space. So, a string like
 * "1997-3-8 1:1:1" will parse successfully. See bug: 1205298
 */

assert.sameValue(new Date("1997-03-08 1:1:1.01").getTime(),
         new Date("1997-03-08T01:01:01.01").getTime());
assert.sameValue(new Date("1997-03-08 11:19:20").getTime(),
         new Date("1997-03-08T11:19:20").getTime());
assert.sameValue(new Date("1997-3-08 11:19:20").getTime(),
         new Date("1997-03-08T11:19:20").getTime());
assert.sameValue(new Date("1997-3-8 11:19:20").getTime(),
         new Date("1997-03-08T11:19:20").getTime());
assert.sameValue(new Date("+001997-3-8 11:19:20").getTime(),
         new Date("1997-03-08T11:19:20").getTime());
assert.sameValue(new Date("+001997-03-8 11:19:20").getTime(),
         new Date("1997-03-08T11:19:20").getTime());
assert.sameValue(new Date("1997-03-08 11:19").getTime(),
         new Date("1997-03-08T11:19").getTime());
assert.sameValue(new Date("1997-03-08 1:19").getTime(),
         new Date("1997-03-08T01:19").getTime());
assert.sameValue(new Date("1997-03-08 1:1").getTime(),
         new Date("1997-03-08T01:01").getTime());
assert.sameValue(new Date("1997-03-08 1:1:01").getTime(),
         new Date("1997-03-08T01:01:01").getTime());
assert.sameValue(new Date("1997-03-08 1:1:1").getTime(),
         new Date("1997-03-08T01:01:01").getTime());
assert.sameValue(new Date("1997-03-08 11").getTime(),
         new Date("1997-03-08T11").getTime()); // Date(NaN)
assert.sameValue(new Date("1997-03-08 11:19:10-07").getTime(),
         new Date("1997-03-08 11:19:10-0700").getTime());
assert.sameValue(new Date("1997-03-08T11:19:10-07").getTime(),
         new Date(NaN).getTime());
assert.sameValue(new Date("1997-03-08T").getTime(),
         new Date(NaN).getTime());
assert.sameValue(new Date("1997-3-8T11:19:20").getTime(),
         new Date(NaN).getTime());
assert.sameValue(new Date("1997-03-8T11:19:20").getTime(),
         new Date(NaN).getTime());
assert.sameValue(new Date("+001997-3-8T11:19:20").getTime(),
         new Date(NaN).getTime());
assert.sameValue(new Date("+001997-3-08T11:19:20").getTime(),
         new Date(NaN).getTime());
assert.sameValue(new Date("1997-03-08T1:19").getTime(),
         new Date(NaN).getTime());
assert.sameValue(new Date("1997-03-08T1:1").getTime(),
         new Date(NaN).getTime());
assert.sameValue(new Date("1997-03-08T1:1:01").getTime(),
         new Date(NaN).getTime());
assert.sameValue(new Date("1997-03-08T1:1:1").getTime(),
         new Date(NaN).getTime());
