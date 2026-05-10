/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
var BUGNUMBER = 518103;
var summary = 'partial flat closures must not reach across funargs';
var actual = "no crash";
var expect = actual;

function Timer(){}
Timer.prototype = { initWithCallback: function (o) {Timer.q.push(o)} };
Timer.q = [];

var later;
var ac = {startSearch: function(q,s,n,o){later=o}};

var bm = {insertBookmark: function(){}, getIdForItemAt: function(){}};

function run_test() {
  var tagIds = [];

  (function doSearch(query) {
    ac.startSearch(query, "", null, {
      onSearchResult: function() {
        var num = tagIds.length;

        var timer = new Timer;
        var next = query.slice(1);
        timer.initWithCallback({ notify: function() { return doSearch(next); } });
      }
    });
  })("title");
}

run_test();
later.onSearchResult();
for (var i in Timer.q)
  Timer.q[i].notify();

assert.sameValue(expect, actual, summary);
