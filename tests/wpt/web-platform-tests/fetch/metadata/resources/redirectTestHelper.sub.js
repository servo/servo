function createVideoElement() {
  let el = document.createElement('video');
  el.src = '/media/movie_5.mp4';
  el.setAttribute('controls', '');
  el.setAttribute('crossorigin', '');
  return el;
}

function createTrack() {
  let el = document.createElement('track');
  el.setAttribute('default', '');
  el.setAttribute('kind', 'captions');
  el.setAttribute('srclang', 'en');
  return el;
}

let secureRedirectURL = 'https://{{host}}:{{ports[https][0]}}/fetch/api/resources/redirect.py?location=';
let insecureRedirectURL = 'http://{{host}}:{{ports[http][0]}}/fetch/api/resources/redirect.py?location=';
let secureTestURL = 'https://{{host}}:{{ports[https][0]}}/fetch/metadata/';
let insecureTestURL = 'http://{{host}}:{{ports[http][0]}}/fetch/metadata/';

// Helper to craft an URL that will go from HTTPS => HTTP => HTTPS to
// simulate us downgrading then upgrading again during the same redirect chain.
function MultipleRedirectTo(partialPath) {
  let finalURL = insecureRedirectURL + encodeURIComponent(secureTestURL + partialPath);
  return secureRedirectURL + encodeURIComponent(finalURL);
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
    let testWindow = window.open(urlHelperMethod('resources/post-to-owner.py?top-level-navigation' + nonce));
    t.add_cleanup(_ => testWindow.close());
    window.addEventListener('message', t.step_func(e => {
      if (e.source != testWindow) {
        return;
      }

      let expectation = { ...expectedResults };
      if (expectation['mode'] != '')
        expectation['mode'] = 'navigate';
      if (expectation['dest'] == 'font')
        expectation['dest'] = 'document';
      assert_header_equals(e.data, expectation, testNamePrefix + ' top level navigation');
      t.done();
    }));
  }, testNamePrefix + ' top level navigation');

  promise_test(t => {
    return new Promise((resolve, reject) => {
      let key = 'embed-https-redirect' + nonce;
      let e = document.createElement('embed');
      e.src = urlHelperMethod('resources/record-header.py?file=' + key);
      e.onload = e => {
        let expectation = { ...expectedResults };
        if (expectation['mode'] != '')
          expectation['mode'] = 'navigate';
        if (expectation['dest'] == 'font')
          expectation['dest'] = 'embed';
        fetch('/fetch/metadata/resources/record-header.py?retrieve=true&file=' + key)
          .then(response => response.text())
          .then(t.step_func(text => assert_header_equals(text, expectation, testNamePrefix + ' embed')))
          .then(resolve)
          .catch(e => reject(e));
      };
      document.body.appendChild(e);
    });
  }, testNamePrefix + ' embed');

  promise_test(t => {
    return new Promise((resolve, reject) => {
      let key = 'object-https-redirect' + nonce;
      let e = document.createElement('object');
      e.data = urlHelperMethod('resources/record-header.py?file=' + key);
      e.onload = e => {
        let expectation = { ...expectedResults };
        if (expectation['mode'] != '')
          expectation['mode'] = 'navigate';
        if (expectation['dest'] == 'font')
          expectation['dest'] = 'object';
        fetch('/fetch/metadata/resources/record-header.py?retrieve=true&file=' + key)
          .then(response => response.text())
          .then(t.step_func(text => assert_header_equals(text, expectation, testNamePrefix + ' object')))
          .then(resolve)
          .catch(e => reject(e));
      };
      document.body.appendChild(e);
    });
  }, testNamePrefix + ' object');

  if (document.createElement('link').relList.supports('preload')) {
    async_test(t => {
      let key = 'preload' + nonce;
      let e = document.createElement('link');
      e.rel = 'preload';
      e.href = urlHelperMethod('resources/record-header.py?file=' + key);
      e.setAttribute('as', 'track');
      e.onload = e.onerror = t.step_func_done(e => {
        let expectation = { ...expectedResults };
        if (expectation['mode'] != '')
          expectation['mode'] = 'cors';
        fetch('/fetch/metadata/resources/record-header.py?retrieve=true&file=' + key)
          .then(t.step_func(response => response.text()))
          .then(t.step_func_done(text => assert_header_equals(text, expectation, testNamePrefix + ' preload')))
          .catch(t.unreached_func());
      });
      document.head.appendChild(e);
    }, testNamePrefix + ' preload');
  }

  promise_test(t => {
    return new Promise((resolve, reject) => {
      let key = 'style-https-redirect' + nonce;
      let e = document.createElement('link');
      e.rel = 'stylesheet';
      e.href = urlHelperMethod('resources/record-header.py?file=' + key);
      e.onload = e => {
        let expectation = { ...expectedResults };
        if (expectation['mode'] != '')
          expectation['mode'] = 'no-cors';
        if (expectation['dest'] == 'font')
          expectation['dest'] = 'style';
        fetch('/fetch/metadata/resources/record-header.py?retrieve=true&file=' + key)
          .then(response => response.text())
          .then(t.step_func(text => assert_header_equals(text, expectation, testNamePrefix + ' stylesheet')))
          .then(resolve)
          .catch(e => reject(e));
      };
      document.body.appendChild(e);
    });
  }, testNamePrefix + ' stylesheet');

  promise_test(t => {
    return new Promise((resolve, reject) => {
      let key = 'track-https-redirect' + nonce;
      let video = createVideoElement();
      let el = createTrack();
      el.src = urlHelperMethod('resources/record-header.py?file=' + key);
      el.onload = t.step_func(_ => {
        let expectation = { ...expectedResults };
        if (expectation['mode'] != '')
          expectation['mode'] = 'cors';
        if (expectation['dest'] == 'font')
          expectation['dest'] = 'track';
        fetch('/fetch/metadata/resources/record-header.py?retrieve=true&file=' + key)
          .then(response => response.text())
          .then(t.step_func(text => assert_header_equals(text, expectation, testNamePrefix + ' track')))
          .then(resolve);
      });
      video.appendChild(el);
      document.body.appendChild(video);
    });
  }, testNamePrefix + ' track');
}
