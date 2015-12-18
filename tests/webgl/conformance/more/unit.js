/*
Unit testing library for the OpenGL ES 2.0 HTML Canvas context
*/

/*
** Copyright (c) 2012 The Khronos Group Inc.
**
** Permission is hereby granted, free of charge, to any person obtaining a
** copy of this software and/or associated documentation files (the
** "Materials"), to deal in the Materials without restriction, including
** without limitation the rights to use, copy, modify, merge, publish,
** distribute, sublicense, and/or sell copies of the Materials, and to
** permit persons to whom the Materials are furnished to do so, subject to
** the following conditions:
**
** The above copyright notice and this permission notice shall be included
** in all copies or substantial portions of the Materials.
**
** THE MATERIALS ARE PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
** EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
** MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
** IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
** CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
** TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
** MATERIALS OR THE USE OR OTHER DEALINGS IN THE MATERIALS.
*/

/* -- plaform specific code -- */

// WebKit
if (window.testRunner && !window.layoutTestController) {
  window.layoutTestController = window.testRunner;
}

if (window.layoutTestController) {
  layoutTestController.overridePreference("WebKitWebGLEnabled", "1");
  layoutTestController.dumpAsText();
  layoutTestController.waitUntilDone();

  // The WebKit testing system compares console output.
  // Because the output of the WebGL Tests is GPU dependent
  // we turn off console messages.
  window.console.log = function() { };
  window.console.error = function() { };

  // RAF doesn't work in LayoutTests. Disable it so the tests will
  // use setTimeout instead.
  window.requestAnimationFrame = undefined;
  window.webkitRequestAnimationFrame = undefined;
}

if (window.internals) {
  window.internals.settings.setWebGLErrorsToConsoleEnabled(false);
}

/* -- end platform specific code --*/
Tests = {
  autorun : true,
  message : null,
  delay : 0,
  autoinit: true,

  startUnit : function(){ return []; },
  setup : function() { return arguments; },
  teardown : function() {},
  endUnit : function() {}
}

var __testSuccess__ = true;
var __testFailCount__ = 0;
var __testLog__;
var __backlog__ = [];

var getUrlOptions = (function() {
  var _urlOptionsParsed = false;
  var _urlOptions = {};
  return function() {
    if (!_urlOptionsParsed) {
      var s = window.location.href;
      var q = s.indexOf("?");
      var e = s.indexOf("#");
      if (e < 0) {
        e = s.length;
      }
      var query = s.substring(q + 1, e);
      var pairs = query.split("&");
      for (var ii = 0; ii < pairs.length; ++ii) {
        var keyValue = pairs[ii].split("=");
        var key = keyValue[0];
        var value = decodeURIComponent(keyValue[1]);
        _urlOptions[key] = value;
      }
      _urlOptionsParsed = true;
    }

    return _urlOptions;
  }
})();

if (typeof quietMode == 'undefined') {
  var quietMode = (function() {
    var _quietModeChecked = false;
    var _isQuiet = false;
    return function() {
      if (!_quietModeChecked) {
        _isQuiet = (getUrlOptions().quiet == 1);
        _quietModeChecked = true;
      }
      return _isQuiet;
    }
  })();
}

Object.toSource = function(a, seen){
  if (a == null) return "null";
  if (typeof a == 'boolean') return a ? "true" : "false";
  if (typeof a == 'string') return '"' + a.replace(/"/g, '\\"') + '"';
  if (a instanceof HTMLElement) return a.toString();
  if (a.width && a.height && a.data) return "[ImageData]";
  if (a instanceof Array) {
    if (!seen) seen = [];
    var idx = seen.indexOf(a);
    if (idx != -1) return '#'+(idx+1)+'#';
    seen.unshift(a);
    var srcs = a.map(function(o){ return Object.toSource(o,seen) });
    var prefix = '';
    idx = seen.indexOf(a);
    if (idx != -1) prefix = '#'+(idx+1)+'=';
    return prefix + '[' + srcs.join(", ") + ']';
  }
  if (typeof a == 'object') {
    if (!seen) seen = [];
    var idx = seen.indexOf(a);
    if (idx != -1) return '#'+(idx+1)+'#';
    seen.unshift(a);
    var members = [];
    var name;
    try {
      for (var i in a) {
        if (i.search(/^[a-zA-Z0-9]+$/) != -1)
          name = i;
        else
          name = '"' + i.replace(/"/g, '\\"') + '"';
        var ai;
        try { ai = a[i]; }
        catch(e) { ai = 'null /*ERROR_ACCESSING*/'; }
        var s = name + ':' + Object.toSource(ai, seen);
        members.push(s);
      }
    } catch (e) {}
    var prefix = '';
    idx = seen.indexOf(a);
    if (idx != -1) prefix = '#'+(idx+1)+'=';
    return prefix + '{' + members.join(", ") + '}'
  }
  if (typeof a == 'function')
    return '('+a.toString().replace(/\n/g, " ").replace(/\s+/g, " ")+')';
  return a.toString();
}

function formatError(e) {
  if (window.console) console.log(e);
  var pathSegs = location.href.toString().split("/");
  var currentDoc = e.lineNumber != null ? pathSegs[pathSegs.length - 1] : null;
  var trace = (e.filename || currentDoc) + ":" + e.lineNumber + (e.trace ? "\n"+e.trace : "");
  return e.message + "\n" + trace;
}

function runTests() {
  var h = document.getElementById('test-status');
  if (h == null) {
    h = document.createElement('h1');
    h.id = 'test-status';
    document.body.appendChild(h);
  }
  h.textContent = "";
  var log = document.getElementById('test-log');
  if (log == null) {
    log = document.createElement('div');
    log.id = 'test-log';
    document.body.appendChild(log);
  }
  while (log.childNodes.length > 0)
    log.removeChild(log.firstChild);

  var setup_args = [];

  if (Tests.startUnit != null) {
    __testLog__ = document.createElement('div');
    try {
      setup_args = Tests.startUnit();
      if (__testLog__.childNodes.length > 0)
        log.appendChild(__testLog__);
    } catch(e) {
      testFailed("startUnit", formatError(e));
      log.appendChild(__testLog__);
      printTestStatus();
      return;
    }
  }

  var testsRun = false;
  var allTestsSuccessful = true;

  for (var i in Tests) {
    if (i.substring(0,4) != "test") continue;
    __testLog__ = document.createElement('div');
    __testSuccess__ = true;
    try {
      doTestNotify (i);
      var args = setup_args;
      if (Tests.setup != null)
        args = Tests.setup.apply(Tests, setup_args);
      Tests[i].apply(Tests, args);
      if (Tests.teardown != null)
        Tests.teardown.apply(Tests, args);
    }
    catch (e) {
      testFailed(i, e.name, formatError(e));
    }
    if (__testSuccess__ == false) {
      ++__testFailCount__;
    }
    var h = document.createElement('h2');
    h.textContent = i;
    __testLog__.insertBefore(h, __testLog__.firstChild);
    log.appendChild(__testLog__);
    allTestsSuccessful = allTestsSuccessful && __testSuccess__ == true;
    reportTestResultsToHarness(__testSuccess__, i);
    doTestNotify (i+"--"+(__testSuccess__?"OK":"FAIL"));
    testsRun = true;
  }

  printTestStatus(testsRun);
  if (Tests.endUnit != null) {
    __testLog__ = document.createElement('div');
    try {
      Tests.endUnit.apply(Tests, setup_args);
      if (__testLog__.childNodes.length > 0)
        log.appendChild(__testLog__);
    } catch(e) {
      testFailed("endUnit", e.name, formatError(e));
      log.appendChild(__testLog__);
    }
  }
  notifyFinishedToHarness(allTestsSuccessful, "finished tests");
}

function doTestNotify(name) {
  //try {
  //  var xhr = new XMLHttpRequest();
  //  xhr.open("GET", "http://localhost:8888/"+name, true);
  //  xhr.send(null);
  //} catch(e) {}
}

function testFailed(assertName, name) {
  var d = document.createElement('div');
  var h = document.createElement('h3');
  var d1 = document.createElement("span");
  h.appendChild(d1);
  d1.appendChild(document.createTextNode("FAIL: "));
  d1.style.color = "red";
  h.appendChild(document.createTextNode(
      name==null ? assertName : name + " (in " + assertName + ")"));
  d.appendChild(h);
  var args = []
  for (var i=2; i<arguments.length; i++) {
    var a = arguments[i];
    var p = document.createElement('p');
    p.style.whiteSpace = 'pre';
    p.textContent = (a == null) ? "null" :
                    (typeof a == 'boolean' || typeof a == 'string') ? a : Object.toSource(a);
    args.push(p.textContent);
    d.appendChild(p);
  }
  __testLog__.appendChild(d);
  __testSuccess__ = false;
  doTestNotify([assertName, name].concat(args).join("--"));
}

function testPassed(assertName, name) {
  if (!quietMode()) {
    var d = document.createElement('div');
    var h = document.createElement('h3');
    var d1 = document.createElement("span");
    h.appendChild(d1);
    d1.appendChild(document.createTextNode("PASS: "));
    d1.style.color = "green";
    h.appendChild(document.createTextNode(
        name==null ? assertName : name + " (in " + assertName + ")"));
    d.appendChild(h);
    var args = []
    for (var i=2; i<arguments.length; i++) {
      var a = arguments[i];
      var p = document.createElement('p');
      p.style.whiteSpace = 'pre';
      p.textContent = (a == null) ? "null" :
                      (typeof a == 'boolean' || typeof a == 'string') ? a : Object.toSource(a);
      args.push(p.textContent);
      d.appendChild(p);
    }
    __testLog__.appendChild(d);
  }
  doTestNotify([assertName, name].concat(args).join("--"));
}

function checkTestSuccess() {
  return __testFailCount__ == 0;
}

window.addEventListener('load', function(){
  for (var i=0; i<__backlog__.length; i++)
    log(__backlog__[i]);
}, false);

function log(msg) {
  var p = document.createElement('p');
  var a = [];
  for (var i=0; i<arguments.length; i++)
    a.push(arguments[i]);
  p.textContent = a.join(", ");
  if (!__testLog__) {
    if (document.body)
      document.body.appendChild(p);
    else
      __backlog__.push(msg);
  } else {
    __testLog__.appendChild(p);
  }
}

function printTestStatus(testsRun) {
  var status = document.getElementById('test-status');
  if (testsRun) {
    status.className = checkTestSuccess() ? 'ok' : 'fail';
    status.textContent = checkTestSuccess() ? "PASS" : "FAIL";
  } else {
    status.className = 'fail';
    status.textContent = "NO TESTS FOUND";
  }
}

function assertFail(name, f) {
  if (f == null) { f = name; name = null; }
  var r = false;
  try { f(); } catch(e) { r=true; }
  if (!r) {
    testFailed("assertFail", name, f);
    return false;
  } else {
    testPassed("assertFail", name, f);
    return true;
  }
}

function assertOk(name, f) {
  if (f == null) { f = name; name = null; }
  var r = false;
  var err;
  try { f(); r=true; } catch(e) { err = e; }
  if (!r) {
    testFailed("assertOk", name, f, err.toString());
    return false;
  } else {
    testPassed("assertOk", name, f);
    return true;
  }
}

function assert(name, v) {
  if (v == null) { v = name; name = null; }
  if (!v) {
    testFailed("assert", name, v);
    return false;
  } else {
    testPassed("assert", name, v);
    return true;
  }
}

function assertProperty(name, v, p) {
  if (p == null) { p = v; v = name; name = p; }
  if (v[p] == null) {
    testFailed("assertProperty", name);
    return false;
  } else {
    testPassed("assertProperty", name);
    return true;
  }
}

function compare(a,b) {
  if (typeof a == 'number' && typeof b == 'number') {
    return a == b;
  } else {
    return Object.toSource(a) == Object.toSource(b);
  }
}

function assertEquals(name, v, p) {
  if (p == null) { p = v; v = name; name = null; }
  if (!compare(v, p)) {
    testFailed("assertEquals", name, v, p);
    return false;
  } else {
    testPassed("assertEquals", name, v, p);
    return true;
  }
}

function assertArrayEquals(name, v, p) {
  if (p == null) { p = v; v = name; name = null; }
  if (!v) {
    testFailed("assertArrayEquals: first array undefined", name, v, p);
    return false;
  }
  if (!p) {
    testFailed("assertArrayEquals: second array undefined", name, v, p);
    return false;
  }
  if (v.length != p.length) {
    testFailed("assertArrayEquals", name, v, p);
    return false;
  }
  for (var ii = 0; ii < v.length; ++ii) {
    if (v[ii] != p[ii]) {
      testFailed("assertArrayEquals", name, v, p);
      return false;
    }
  }
  testPassed("assertArrayEquals", name, v, p);
  return true;
}

function assertArrayEqualsWithEpsilon(name, v, p, l) {
  if (l == null) { l = p; p = v; v = name; name = null; }
  if (!v) {
    testFailed("assertArrayEqualsWithEpsilon: first array undefined", name, v, p);
    return false;
  }
  if (!p) {
    testFailed("assertArrayEqualsWithEpsilon: second array undefined", name, v, p);
    return false;
  }
  if (!l) {
    testFailed("assertArrayEqualsWithEpsilon: limit array undefined", name, v, p);
    return false;
  }
  if (v.length != p.length) {
    testFailed("assertArrayEqualsWithEpsilon", name, v, p, l);
    return false;
  }
  if (v.length != l.length) {
    testFailed("assertArrayEqualsWithEpsilon", name, v, p, l);
    return false;
  }
  for (var ii = 0; ii < v.length; ++ii) {
    if (Math.abs(v[ii]- p[ii])>l[ii]) {
      testFailed("assertArrayEqualsWithEpsilon", name, v, p, l);
      return false;
    }
  }
  testPassed("assertArrayEqualsWithEpsilon", name, v, p, l);
  return true;
}

function assertNotEquals(name, v, p) {
  if (p == null) { p = v; v = name; name = null; }
  if (compare(v, p)) {
    testFailed("assertNotEquals", name, v, p)
    return false;
  } else {
    testPassed("assertNotEquals", name, v, p)
    return true;
  }
}

function time(elementId, f) {
    var s = document.getElementById(elementId);
    var t0 = new Date().getTime();
    f();
    var t1 = new Date().getTime();
    s.textContent = 'Elapsed: '+(t1-t0)+' ms';
}

function randomFloat () {
    // note that in fuzz-testing, this can used as the size of a buffer to allocate.
    // so it shouldn't return astronomic values. The maximum value 10000000 is already quite big.
    var fac = 1.0;
    var r = Math.random();
    if (r < 0.25)
        fac = 10;
    else if (r < 0.4)
        fac = 100;
    else if (r < 0.5)
        fac = 1000;
    else if (r < 0.6)
        fac = 100000;
    else if (r < 0.7)
        fac = 10000000;
    else if (r < 0.8)
        fac = NaN;
    return -0.5*fac + Math.random() * fac;
}
function randomFloatFromRange(lo, hi) {
  var r = Math.random();
  if (r < 0.05)
    return lo;
  else if (r > 0.95)
    return hi;
  else
    return lo + Math.random()*(hi-lo);
}
function randomInt (sz) {
  if (sz != null)
    return Math.floor(Math.random()*sz);
  else
    return Math.floor(randomFloat());
}
function randomIntFromRange(lo, hi) {
  return Math.floor(randomFloatFromRange(lo, hi));
}
function randomLength () {
    var l = Math.floor(Math.random() * 256);
    if (Math.random < 0.5) l = l / 10;
    if (Math.random < 0.3) l = l / 10;
    return l;
}
function randomSmallIntArray () {
    var l = randomLength();
    var s = new Array(l);
    for (var i=0; i<l; i++)
        s[i] = Math.floor(Math.random() * 256)-1;
    return s;
}
function randomFloatArray () {
    var l = randomLength();
    var s = new Array(l);
    for (var i=0; i<l; i++)
        s[i] = randomFloat();
    return s;
}
function randomIntArray () {
    var l = randomLength();
    var s = new Array(l);
    for (var i=0; i<l; i++)
        s[i] = randomFloat();
    return s;
}
function randomMixedArray () {
    var l = randomLength();
    var s = new Array(l);
    for (var i=0; i<l; i++)
        s[i] = randomNonArray();
    return s;
}
function randomArray () {
    var r = Math.random();
    if (r < 0.3)
        return randomFloatArray();
    else if (r < 0.6)
        return randomIntArray();
    else if (r < 0.8)
        return randomSmallIntArray();
    else
        return randomMixedArray();
}
function randomString () {
    return String.fromCharCode.apply(String, randomSmallIntArray());
}
function randomGLConstant () {
    return GLConstants[Math.floor(Math.random() * GLConstants.length)];
}

function randomNonArray() {
    var r = Math.random();
    if (r < 0.25) {
        return randomFloat();
    } else if (r < 0.6) {
        return randomInt();
    } else if (r < 0.7) {
        return (r < 0.65);
    } else if (r < 0.87) {
        return randomString();
    } else if (r < 0.98) {
        return randomGLConstant();
    } else {
        return null;
    }
}

function generateRandomArg(pos, count) {
    if (pos == 0 && Math.random() < 0.5)
        return randomGLConstant();
    if (pos == count-1 && Math.random() < 0.25)
        if (Math.random() < 0.5)
            return randomString();
        else
            return randomArray();
    var r = Math.random();
    if (r < 0.25) {
        return randomFloat();
    } else if (r < 0.6) {
        return randomInt();
    } else if (r < 0.7) {
        return (r < 0.65);
    } else if (r < 0.77) {
        return randomString();
    } else if (r < 0.84) {
        return randomArray();
    } else if (r < 0.98) {
        return randomGLConstant();
    } else {
        return null;
    }
}


function generateRandomArgs(count) {
    var arr = new Array(count);
    for (var i=0; i<count; i++)
        arr[i] = generateRandomArg(i, count);
    return arr;
}

// qc (arg1gen, arg2gen, ..., predicate)
// qc (randomString, randomInt, randomInt, function(s,i,j){ s.substring(i,j) })
function qc() {
}

GLConstants = [
1,
0x00000100,
0x00000400,
0x00004000,
0x0000,
0x0001,
0x0002,
0x0003,
0x0004,
0x0005,
0x0006,
0,
1,
0x0300,
0x0301,
0x0302,
0x0303,
0x0304,
0x0305,
0x0306,
0x0307,
0x0308,
0x8006,
0x8009,
0x8009,
0x883D,
0x800A,
0x800B,
0x80C8,
0x80C9,
0x80CA,
0x80CB,
0x8001,
0x8002,
0x8003,
0x8004,
0x8005,
0x8892,
0x8893,
0x8894,
0x8895,
0x88E0,
0x88E4,
0x88E8,
0x8764,
0x8765,
0x8626,
0x0404,
0x0405,
0x0408,
0x0DE1,
0x0B44,
0x0BE2,
0x0BD0,
0x0B90,
0x0B71,
0x0C11,
0x8037,
0x809E,
0x80A0,
0,
0x0500,
0x0501,
0x0502,
0x0505,
0x0900,
0x0901,
0x0B21,
0x846D,
0x846E,
0x0B45,
0x0B46,
0x0B70,
0x0B72,
0x0B73,
0x0B74,
0x0B91,
0x0B92,
0x0B94,
0x0B95,
0x0B96,
0x0B97,
0x0B93,
0x0B98,
0x8800,
0x8801,
0x8802,
0x8803,
0x8CA3,
0x8CA4,
0x8CA5,
0x0BA2,
0x0C10,
0x0C22,
0x0C23,
0x0CF5,
0x0D05,
0x0D33,
0x0D3A,
0x0D50,
0x0D52,
0x0D53,
0x0D54,
0x0D55,
0x0D56,
0x0D57,
0x2A00,
0x8038,
0x8069,
0x80A8,
0x80A9,
0x80AA,
0x80AB,
0x86A2,
0x86A3,
0x1100,
0x1101,
0x1102,
0x8192,
0x1400,
0x1401,
0x1402,
0x1403,
0x1404,
0x1405,
0x1406,
0x140C,
0x1902,
0x1906,
0x1907,
0x1908,
0x1909,
0x190A,
0x8033,
0x8034,
0x8363,
0x8B30,
0x8B31,
0x8869,
0x8DFB,
0x8DFC,
0x8B4D,
0x8B4C,
0x8872,
0x8DFD,
0x8B4F,
0x8B80,
0x8B82,
0x8B83,
0x8B85,
0x8B86,
0x8B87,
0x8B89,
0x8B8A,
0x8B8C,
0x8B8D,
0x0200,
0x0201,
0x0202,
0x0203,
0x0204,
0x0205,
0x0206,
0x0207,
0x1E00,
0x1E01,
0x1E02,
0x1E03,
0x150A,
0x8507,
0x8508,
0x1F00,
0x1F01,
0x1F02,
0x1F03,
0x2600,
0x2601,
0x2700,
0x2701,
0x2702,
0x2703,
0x2800,
0x2801,
0x2802,
0x2803,
0x1702,
0x8513,
0x8514,
0x8515,
0x8516,
0x8517,
0x8518,
0x8519,
0x851A,
0x851C,
0x84C0,
0x84C1,
0x84C2,
0x84C3,
0x84C4,
0x84C5,
0x84C6,
0x84C7,
0x84C8,
0x84C9,
0x84CA,
0x84CB,
0x84CC,
0x84CD,
0x84CE,
0x84CF,
0x84D0,
0x84D1,
0x84D2,
0x84D3,
0x84D4,
0x84D5,
0x84D6,
0x84D7,
0x84D8,
0x84D9,
0x84DA,
0x84DB,
0x84DC,
0x84DD,
0x84DE,
0x84DF,
0x84E0,
0x2901,
0x812F,
0x8370,
0x8B50,
0x8B51,
0x8B52,
0x8B53,
0x8B54,
0x8B55,
0x8B56,
0x8B57,
0x8B58,
0x8B59,
0x8B5A,
0x8B5B,
0x8B5C,
0x8B5E,
0x8B60,
0x8622,
0x8623,
0x8624,
0x8625,
0x886A,
0x8645,
0x889F,
0x8B9A,
0x8B9B,
0x8B81,
0x8B84,
0x8B88,
0x8DFA,
0x8DF8,
0x8DF9,
0x8DF0,
0x8DF1,
0x8DF2,
0x8DF3,
0x8DF4,
0x8DF5,
0x8D40,
0x8D41,
0x8056,
0x8057,
0x8D62,
0x81A5,
0x1901,
0x8D48,
0x8D42,
0x8D43,
0x8D44,
0x8D50,
0x8D51,
0x8D52,
0x8D53,
0x8D54,
0x8D55,
0x8CD0,
0x8CD1,
0x8CD2,
0x8CD3,
0x8CE0,
0x8D00,
0x8D20,
0,
0x8CD5,
0x8CD6,
0x8CD7,
0x8CD9,
0x8CDD,
0x8CA6,
0x8CA7,
0x84E8,
0x0506,
0x809D
];

function reportTestResultsToHarness(success, msg) {
  if (window.parent.webglTestHarness) {
    window.parent.webglTestHarness.reportResults(window.location.pathname, success, msg);
  }
}

function notifyFinishedToHarness() {
  if (window.parent.webglTestHarness) {
    window.parent.webglTestHarness.notifyFinished(window.location.pathname);
  }
}

function initTests() {
  if (Tests.message != null) {
    var h = document.getElementById('test-message');
    if (h == null) {
      h = document.createElement('p');
      h.id = 'test-message';
      document.body.insertBefore(h, document.body.firstChild);
    }
    h.textContent = Tests.message;
  }
  if (Tests.autorun) {
    runTests();
  } else {
    var h = document.getElementById('test-run');
    if (h == null) {
      h = document.createElement('input');
      h.type = 'submit';
      h.value = "Run tests";
      h.addEventListener('click', function(ev){
        runTests();
        ev.preventDefault();
      }, false);
      h.id = 'test-run';
      document.body.insertBefore(h, document.body.firstChild);
    }
    h.textContent = Tests.message;
  }

}

window.addEventListener('load', function(){
  if (Tests.autoinit) {
    // let the browser hopefully finish updating the gl canvas surfaces if we are given a delay
    if (Tests.delay)
      setTimeout(initTests, Tests.delay);
    else
      initTests()
  }
}, false);

