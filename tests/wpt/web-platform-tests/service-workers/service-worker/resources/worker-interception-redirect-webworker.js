// This is the (shared or dedicated) worker file for the
// worker-interception-redirect test. It should be served by the corresponding
// .py file instead of being served directly.
//
// This file is served from both resources/*webworker.py and
// resources/scope2/*webworker.py, hence some of the complexity
// below about paths.
const resources_url = new URL("/service-workers/service-worker/resources/",
                              self.location);

// This greeting text is meant to be injected by the Python script that serves
// this file, to indicate how the script was served (from network or from
// service worker).
//
// We can't just use a sub pipe and name this file .sub.js since we want
// to serve the file from multiple URLs (see above).
let greeting = '%GREETING_TEXT%';
if (!greeting)
  greeting = 'the shared worker script was served from network';

// Call importScripts() which fills |echo_output| with a string indicating
// whether a service worker intercepted the importScripts() request.
let echo_output;
const import_scripts_msg = encodeURIComponent(
    'importScripts: served from network');
const import_scripts_url =
    new URL(`import-scripts-echo.py?msg=${import_scripts_msg}`, resources_url);
importScripts(import_scripts_url);
const import_scripts_greeting = echo_output;

self.onconnect = async function(e) {
  const port = e.ports[0];
  port.start();
  port.postMessage(greeting);

  port.postMessage(import_scripts_greeting);

  const fetch_url = new URL('simple.txt', resources_url);
  const response = await fetch(fetch_url);
  const text = await response.text();
  port.postMessage('fetch(): ' + text);

  port.postMessage(self.location.href);
};
