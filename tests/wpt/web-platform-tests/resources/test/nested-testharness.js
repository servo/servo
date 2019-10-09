'use strict';

/**
 * Execute testharness.js and one or more scripts in an iframe. Report the
 * results of the execution.
 *
 * @param {...function|...string} bodies - a function body. If specified as a
 *                                         function object, it will be
 *                                         serialized to a string using the
 *                                         built-in
 *                                         `Function.prototype.toString` prior
 *                                         to inclusion in the generated
 *                                         iframe.
 *
 * @returns {Promise} eventual value describing the result of the test
 *                    execution; the summary object has two properties:
 *                    `harness` (a string describing the harness status) and
 *                    `tests` (an object whose "own" property names are the
 *                    titles of the defined sub-tests and whose associated
 *                    values are the subtest statuses).
 */
function makeTest(...bodies) {
  const closeScript = '<' + '/script>';
  let src = `
<!DOCTYPE HTML>
<html>
<head>
<title>Document title</title>
<script src="/resources/testharness.js?${Math.random()}">${closeScript}
</head>

<body>
<div id="log"></div>`;
  bodies.forEach((body) => {
    src += '<script>(' + body + ')();' + closeScript;
  });

  const iframe = document.createElement('iframe');

  document.body.appendChild(iframe);
  iframe.contentDocument.write(src);

  return new Promise((resolve) => {
    window.addEventListener('message', function onMessage(e) {
      if (e.source !== iframe.contentWindow) {
        return;
      }
      if (!e.data || e.data.type !=='complete') {
        return;
      }
      window.removeEventListener('message', onMessage);
      resolve(e.data);
    });

    iframe.contentDocument.close();
  }).then(({ tests, status }) => {
    const summary = {
      harness: getEnumProp(status, status.status),
      tests: {}
    };

    tests.forEach((test) => {
      summary.tests[test.name] = getEnumProp(test, test.status);
    });

    return summary;
  });
}

function getEnumProp(object, value) {
  for (let property in object) {
    if (!/^[A-Z]+$/.test(property)) {
      continue;
    }

    if (object[property] === value) {
      return property;
    }
  }
}
