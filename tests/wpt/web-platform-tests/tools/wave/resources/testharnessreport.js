/* global add_completion_callback */
/* global setup */

/*
 * This file is intended for vendors to implement code needed to integrate
 * testharness.js tests with their own test systems.
 *
 * Typically test system integration will attach callbacks when each test has
 * run, using add_result_callback(callback(test)), or when the whole test file
 * has completed, using
 * add_completion_callback(callback(tests, harness_status)).
 *
 * For more documentation about the callback functions and the
 * parameters they are called with see testharness.js
 */

/*
 * If the query parameter token is available means that the test was loaded by 
 * the WAVE test runner and the results need to be reported to the server using
 * the provided token to identify the session associated this token.  
 */
console.log("ARDVAARD")
if (location.search && location.search.indexOf("token=") != -1) {
   var __WAVE__HOSTNAME = location.hostname;
   var __WAVE__PORT = location.port;
   var __WAVE__PROTOCOL = location.protocol.replace(/:/, "");
   var __WAVE__QUERY = location.search;
   if (!__WAVE__QUERY) __WAVE__QUERY = "?";
   var match = __WAVE__QUERY.match(/https_port=(\d+)/);
   var __HTTPS_PORT = parseInt(match && match[1] ? match[1] : 443);
   match = __WAVE__QUERY.match(/timeout=(\d+)/);
   var __WAVE__TIMEOUT = parseInt(match && match[1] ? match[1] : 65000);
   match = __WAVE__QUERY.match(/web_root=(.+)/);
   var __WAVE__WEB_ROOT = match && match[1] ? match[1] : "/wave/";
    console.log("\n\n\n\n\n")
    console.log(match)
    console.log(__WAVE__WEB_ROOT)
   match = __WAVE__QUERY.match(/token=([^&]+)/);
   var __WAVE__TOKEN = match ? match[1] : null;
   var __WAVE__TEST = location.pathname;
   var nextUrl = null;
   var resultSent = false;
   var screenConsole;

   try {
      var documentRoot = document.body ? document.body : document.documentElement;
      documentRoot.style["background-color"] = "#FFF";
      window.open = function () {
         logToConsole(
            "window.open() is overridden in testharnessreport.js and has not effect"
         );
         var dummyWin = {
            close: function () {
               logToConsole(
                  "dummyWindow.close() in testharnessreport.js and has not effect"
               );
            }
         };
         return dummyWin;
      };
      window.close = function () {
         logToConsole(
            "window.close() is overridden in testharnessreport.js and has not effect"
         );
      };
   } catch (err) {}

   setTimeout(function () {
      loadNext();
   }, __WAVE__TIMEOUT);

   function logToConsole() {
      var text = "";
      for (var i = 0; i < arguments.length; i++) {
         text += arguments[i] + " ";
      }
      if (console && console.log) {
         console.log(text);
      }
      if (screenConsole) {
         try {
            text = text.replace(/ /gm, "&nbsp;");
            text = text.replace(/\n/gm, "<br/>");
            screenConsole.innerHTML += "<br/>" + text;
         } catch (error) {
            screenConsole.innerText += "\n" + text;
         }
      }
   }

   function dump_and_report_test_results(tests, status) {
      var results_element = document.createElement("script");
      results_element.type = "text/json";
      results_element.id = "__testharness__results__";
      var test_results = tests.map(function (x) {
         return {
            name: x.name,
            status: x.status,
            message: x.message,
            stack: x.stack
         };
      });
      var data = {
         test: window.location.href,
         tests: test_results,
         status: status.status,
         message: status.message,
         stack: status.stack
      };
      results_element.textContent = JSON.stringify(data);

      // To avoid a HierarchyRequestError with XML documents, ensure that 'results_element'
      // is inserted at a location that results in a valid document.
      var parent = document.body ?
         document.body // <body> is required in XHTML documents
         :
         document.documentElement; // fallback for optional <body> in HTML5, SVG, etc.

      parent.appendChild(results_element);

      screenConsole = document.createElement("div");
      screenConsole.setAttribute("id", "console");
      screenConsole.setAttribute("style", "font-family: monospace; padding: 5px");
      parent.appendChild(screenConsole);
      window.onerror = logToConsole;

      finishWptTest(data);
   }

   function finishWptTest(data) {
      logToConsole("Creating result ...");
      data.test = __WAVE__TEST;
      createResult(
         __WAVE__TOKEN,
         data,
         function () {
            logToConsole("Result created.");
            loadNext();
         },
         function () {
            logToConsole("Failed to create result.");
            logToConsole("Trying alternative method ...");
            createResultAlt(__WAVE__TOKEN, data);
         }
      );
   }

   function loadNext() {
      logToConsole("Loading next test ...");
      readNextTest(
         __WAVE__TOKEN,
         function (url) {
            logToConsole("Redirecting to " + url);
            location.href = url;
         },
         function () {
            logToConsole("Could not load next test.");
            logToConsole("Trying alternative method ...");
            readNextAlt(__WAVE__TOKEN);
         }
      );
   }

   function readNextTest(token, onSuccess, onError) {
      sendRequest(
         "GET",
         "api/tests/" + token + "/next",
         null,
         null,
         function (response) {
            var jsonObject = JSON.parse(response);
            onSuccess(jsonObject.next_test);
         },
         onError
      );
   }

   function readNextAlt(token) {
      location.href = getWaveUrl("next.html?token=" + token);
   }

   function createResult(token, result, onSuccess, onError) {
      sendRequest(
         "POST",
         "api/results/" + token, {
            "Content-Type": "application/json"
         },
         JSON.stringify(result),
         function () {
            onSuccess();
         },
         onError
      );
   }

   function createResultAlt(token, result) {
      location.href = __WAVE__WEB_ROOT + "submitresult.html" +
         "?token=" + token +
         "&result=" + encodeURIComponent(JSON.stringify(result));
   }

   function sendRequest(method, uri, headers, data, onSuccess, onError) {
      var url = getWaveUrl(uri);
      var xhr = new XMLHttpRequest();
      xhr.addEventListener("load", function () {
         onSuccess(xhr.response);
      });
      xhr.addEventListener("error", function () {
         if (onError) onError();
      });
      logToConsole("Sending", method, 'request to "' + url + '"');
      xhr.open(method, url, true);
      if (headers) {
         for (var header in headers) {
            xhr.setRequestHeader(header, headers[header]);
         }
      }
      xhr.send(data);
   }

   function getWaveUrl(uri) {
      var url = __WAVE__WEB_ROOT + uri;
      console.log(url)
      return url;
   }

   add_completion_callback(dump_and_report_test_results);

} else {
   function dump_test_results(tests, status) {
      var results_element = document.createElement("script");
      results_element.type = "text/json";
      results_element.id = "__testharness__results__";
      var test_results = tests.map(function (x) {
         return {
            name: x.name,
            status: x.status,
            message: x.message,
            stack: x.stack
         }
      });
      var data = {
         test: window.location.href,
         tests: test_results,
         status: status.status,
         message: status.message,
         stack: status.stack
      };
      results_element.textContent = JSON.stringify(data);

      // To avoid a HierarchyRequestError with XML documents, ensure that 'results_element'
      // is inserted at a location that results in a valid document.
      var parent = document.body
         ? document.body // <body> is required in XHTML documents
         : document.documentElement; // fallback for optional <body> in HTML5, SVG, etc.

      parent.appendChild(results_element);
   }

   add_completion_callback(dump_test_results);

   /* If the parent window has a testharness_properties object,
    * we use this to provide the test settings. This is used by the
    * default in-browser runner to configure the timeout and the
    * rendering of results
    */
   try {
      if (window.opener && "testharness_properties" in window.opener) {
         /* If we pass the testharness_properties object as-is here without
          * JSON stringifying and reparsing it, IE fails & emits the message
          * "Could not complete the operation due to error 80700019".
          */
         setup(JSON.parse(JSON.stringify(window.opener.testharness_properties)));
      }
   } catch (e) {}
   // vim: set expandtab shiftwidth=4 tabstop=4:
}
