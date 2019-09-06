function createVideoElement() {
  let el = document.createElement('video');
  el.src = "/media/movie_5.mp4";
  el.setAttribute("controls", "");
  el.setAttribute("crossorigin", "");
  return el;
}

function createTrack() {
  let el = document.createElement("track");
  el.setAttribute("default", "");
  el.setAttribute("kind", "captions");
  el.setAttribute("srclang", "en");
  return el;
}

let secureRedirectURL = "https://{{host}}:{{ports[https][0]}}/fetch/api/resources/redirect.py?location=";
let insecureRedirectURL = "http://{{host}}:{{ports[http][0]}}/fetch/api/resources/redirect.py?location=";
let secureTestURL = "https://{{host}}:{{ports[https][0]}}/fetch/sec-metadata/";
let insecureTestURL = "http://{{host}}:{{ports[http][0]}}/fetch/sec-metadata/";

// Helper to craft an URL that will go from HTTPS => HTTP => HTTPS to
// simulate us downgrading then upgrading again during the same redirect chain.
function MultipleRedirectTo(partialPath) {
  let finalURL = insecureRedirectURL + encodeURIComponent(secureTestURL + partialPath);
  return insecureRedirectURL + encodeURIComponent(finalURL);
}

// Helper to craft an URL that will go from HTTP => HTTPS to simulate upgrading a
// given request.
function upgradeRedirectTo(partialPath) {
  return insecureRedirectURL + encodeURIComponent(secureTestURL + partialPath);
}

// Helper to craft an URL that will go from HTTPS => HTTP to simulate downgrading a
// given request.
function downgradeRedirectTo(partialPath) {
  return secureRedirectURL + encodeURIComponent(insecureTestURL + partialPath);
}

// Helper to run common redirect test cases that don't require special setup on
// the test page itself.
function RunCommonRedirectTests(testNamePrefix, urlHelperMethod, expectedResults) {
  async_test(t => {
    let i = document.createElement('iframe');
    i.src = urlHelperMethod("resources/post-to-owner.py");
    window.addEventListener('message', t.step_func(e => {
      if (e.source != i.contentWindow) {
        return;
      }

      assert_header_equals(e.data, expectedResults);
      t.done();
    }));

    document.body.appendChild(i);
  }, testNamePrefix + " iframe => No headers");

  async_test(t => {
    let testWindow = window.open(urlHelperMethod("resources/post-to-owner.py"));
    t.add_cleanup(_ => testWindow.close());
    window.addEventListener('message', t.step_func(e => {
      if (e.source != testWindow) {
        return;
      }

      assert_header_equals(e.data, expectedResults);
      t.done();
    }));
  }, testNamePrefix + " top level navigation => No headers");

  promise_test(t => {
    return new Promise((resolve, reject) => {
      let key = "embed-redirect-redirect" + nonce;

      let e = document.createElement('embed');
      e.src = urlHelperMethod("resources/record-header.py?file=" + key);
      e.onload = e => {
        fetch("/fetch/sec-metadata/resources/record-header.py?retrieve=true&file=" + key)
          .then(response => response.text())
          .then(t.step_func(text => assert_header_equals(text, expectedResults)))
          .then(_ => resolve())
          .catch(e => reject(e));
        };

      document.body.appendChild(e);
    });
  }, testNamePrefix + " embed => No headers");

  promise_test(t => {
    let key = "fetch-redirect" + nonce;
    return fetch(urlHelperMethod("resources/echo-as-json.py?" + key))
      .then(r => r.json())
      .then(j => {assert_header_equals(j, expectedResults);});
  }, testNamePrefix + " fetch() api => No headers");

  promise_test(t => {
    return new Promise((resolve, reject) => {
      let key = "object-https-redirect" + nonce;
      let e = document.createElement('object');
      e.data = urlHelperMethod("resources/record-header.py?file=" + key);
      e.onload = e => {
      fetch("/fetch/sec-metadata/resources/record-header.py?retrieve=true&file=" + key)
        .then(response => response.text())
        .then(t.step_func(text => assert_header_equals(text, expectedResults)))
        .then(_ => resolve())
        .catch(e => reject(e));
      };
      document.body.appendChild(e);
    });
  }, testNamePrefix + " object => No headers");

  if (document.createElement('link').relList.supports('prefetch')) {
    async_test(t => {
      let key = "prefetch" + nonce;
      let e = document.createElement('link');
      e.rel = "prefetch";
      e.href = urlHelperMethod("resources/record-header.py?file=" + key) + "&simple=true";
      e.onload = t.step_func(e => {
        fetch("/fetch/sec-metadata/resources/record-header.py?retrieve=true&file=" + key)
          .then(t.step_func(response => response.text()))
          .then(t.step_func_done(text => assert_header_equals(text, expectedResults)))
          .catch(t.unreached_func("Fetching and verifying the results should succeed."));
      });
      e.onerror = t.unreached_func();
      document.head.appendChild(e);
    }, testNamePrefix + " prefetch => No headers");
  }

  if (document.createElement('link').relList.supports('preload')) {
    async_test(t => {
      let key = "preload" + nonce;
      let e = document.createElement('link');
      e.rel = "preload";
      e.href = urlHelperMethod("resources/record-header.py?file=" + key);
      e.setAttribute("as", "track");
      e.onload = e.onerror = t.step_func_done(e => {
        fetch("/fetch/sec-metadata/resources/record-header.py?retrieve=true&file=" + key)
          .then(t.step_func(response => response.text()))
          .then(t.step_func(text => assert_header_equals(text, expectedResults)))
          .then(t.step_func_done(_ => resolve()))
          .catch(t.unreached_func());
      });
      document.head.appendChild(e);
    }, testNamePrefix + " preload => No headers");
  }

  promise_test(t => {
    return new Promise((resolve, reject) => {
      let key = "style-https-redirect" + nonce;
      let e = document.createElement('link');
      e.rel = "stylesheet";
      e.href = urlHelperMethod("resources/record-header.py?file=" + key);
      e.onload = e => {
        fetch("/fetch/sec-metadata/resources/record-header.py?retrieve=true&file=" + key)
          .then(response => response.text())
          .then(t.step_func(text => assert_header_equals(text, expectedResults)))
          .then(_ => resolve())
          .catch(e => reject(e));
      };
      document.body.appendChild(e);
    });
  }, testNamePrefix + " stylesheet => No headers");

  promise_test(t => {
    return new Promise((resolve, reject) => {
      let key = "track-https-redirect" + nonce;
      let video = createVideoElement();
      let el = createTrack();
      el.src = urlHelperMethod("resources/record-header.py?file=" + key);
      el.onload = t.step_func(_ => {
        fetch("/fetch/sec-metadata/resources/record-header.py?retrieve=true&file=" + key)
          .then(response => response.text())
          .then(t.step_func(text => assert_header_equals(text, expectedResults)))
          .then(_ => resolve());
      });
      video.appendChild(el);
      document.body.appendChild(video);
    });
  }, testNamePrefix + " track => No headers");
}
