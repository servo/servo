// runTestsFromIframe() is used in the top-level HTML to set cookies and then
// start actual tests in iframe.
function runTestsFromIframe(iframe_url) {
  const setSameOriginCookiePromise = fetch(
    '/cookies/resources/set-cookie.py?name=same&path=/html/semantics/scripting-1/the-script-element/module/',
    {
      mode: 'no-cors',
      credentials: 'include',
    });
  const setCrossOriginCookiePromise = fetch(
    'http://{{domains[www2]}}:{{ports[http][0]}}/cookies/resources/set-cookie.py?name=cross&path=/html/semantics/scripting-1/the-script-element/module/',
    {
      mode: 'no-cors',
      credentials: 'include',
    });
  const windowLoadPromise = new Promise(resolve => {
    window.addEventListener('load', () => {
      resolve();
    });
  });

  const iframe = document.createElement('iframe');
  Promise.all([setSameOriginCookiePromise,
               setCrossOriginCookiePromise,
               windowLoadPromise]).then(() => {
    iframe.src = iframe_url;
    document.body.appendChild(iframe);
    fetch_tests_from_window(iframe.contentWindow);
  });
}

// The functions below are used from tests within the iframe.

let testNumber = 0;

// importFunc and setTimeoutFunc is used to make the active script at the time
// of import() to be the script elements that call `runTest()`,
// NOT this script defining runTest().

function runTest(importFunc, origin, expected, source) {
  let url;
  let description;
  if (origin === 'same') {
    url = "./check-cookie.py";
    description = "Same-origin dynamic import from " + source;
  } else {
    url = "http://{{domains[www2]}}:{{ports[http][0]}}/html/semantics/scripting-1/the-script-element/module/resources/check-cookie.py";
    description = "Cross-origin dynamic import from " + source;
  }
  promise_test(() => {
    const id = "test" + testNumber;
    testNumber += 1;
    return importFunc(url + "?id=" + id + "&cookieName=" + origin + "&origin=" + location.origin)
      .then(() => {
          assert_equals(window[id], expected, "cookie");
        });
  }, description);
}

function setTimeoutWrapper(setTimeoutFunc) {
  return url => {
    return new Promise(resolve => {
      window.resolve = resolve;
      setTimeoutFunc(`import("${url}").then(window.resolve)`);
    });
  };
}
