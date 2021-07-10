"use strict";

importScripts("/resources/testharness.js");

// CORS-same-origin
self.testName = "same-origin importScripts()";
self.baseUrlSanitized = false;
importScripts("../beta/redirect.py?location=http://{{host}}:{{ports[http][0]}}/html/semantics/scripting-1/the-script-element/module/dynamic-import/gamma/base-url.sub.js");

// CORS-cross-origin
self.testName = "cross-origin importScripts()";
self.baseUrlSanitized = true;
importScripts("../beta/redirect.py?location=http://{{hosts[alt][]}}:{{ports[http][0]}}/html/semantics/scripting-1/the-script-element/module/dynamic-import/gamma/base-url.sub.js");

done();
