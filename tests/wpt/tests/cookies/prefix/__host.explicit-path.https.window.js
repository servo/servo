// META: title=The "__Host-" prefix requires an explicit Path attribute of "/"
// META: timeout=long

'use strict';

// A "__Host-" prefixed cookie has to have been set with a Path attribute whose
// value is "/". That is not the same as ending up with a path of "/": a cookie
// set without a Path attribute from a URL whose path has a single segment gets a
// default path of "/" as well, and has to be rejected all the same.
//
// Reaching that case needs a document whose URL has a single path segment, so
// that its default cookie path is "/". "/" itself is the only such URL that
// wptserve does not redirect, and a document there is same-origin, so cookies can
// be set and read through it. HTTP is not covered for want of a handler at that
// depth.
//
// https://httpwg.org/http-extensions/draft-ietf-httpbis-layered-cookies.html#sane-set-cookie

const NAME_PREFIX = 'explicit-path-';

// A document whose default cookie path is "/".
function loadRootDocument() {
  return new Promise(resolve => {
    const iframe = document.createElement('iframe');
    iframe.style = 'display: none';
    iframe.addEventListener('load', () => resolve(iframe), {once: true});
    iframe.src = '/';
    document.body.appendChild(iframe);
  });
}

function cookieValue(cookieString, name) {
  for (const pair of cookieString.split('; ')) {
    const index = pair.indexOf('=');
    if (index !== -1 && pair.substring(0, index) === name) {
      return pair.substring(index + 1);
    }
  }
  return null;
}

// A "__Host-" prefixed cookie can only be expired by a Set-Cookie that satisfies
// the prefix itself, so the cleanup below carries Secure and Path=/ too.
function rootDocumentTest({name, suffix, attributes, stored}) {
  const cookieName = `__Host-${NAME_PREFIX}${suffix}`;
  promise_test(async t => {
    t.add_cleanup(() => {
      document.cookie = `${cookieName}=; Secure; Path=/; Max-Age=0`;
    });

    const iframe = await loadRootDocument();
    t.add_cleanup(() => iframe.remove());
    const doc = iframe.contentWindow.document;
    assert_equals(iframe.contentWindow.location.pathname, '/',
                  'The document has a single path segment');

    doc.cookie = `${cookieName}=1; ${attributes}`;
    assert_equals(cookieValue(doc.cookie, cookieName), stored ? '1' : null);
  }, name);
}

// Establishes the premise: a cookie set from this document without a Path
// attribute has a default path of "/", so it reaches a path outside this test's
// own directory.
promise_test(async t => {
  const cookieName = `${NAME_PREFIX}defaultpath`;
  t.add_cleanup(() => {
    document.cookie = `${cookieName}=; Path=/; Max-Age=0`;
  });

  const root = await loadRootDocument();
  t.add_cleanup(() => root.remove());
  root.contentWindow.document.cookie = `${cookieName}=1`;

  const elsewhere = await new Promise(resolve => {
    const iframe = document.createElement('iframe');
    iframe.style = 'display: none';
    iframe.addEventListener('load', () => resolve(iframe), {once: true});
    iframe.src = '/common/blank.html';
    document.body.appendChild(iframe);
  });
  t.add_cleanup(() => elsewhere.remove());

  assert_equals(
      cookieValue(elsewhere.contentWindow.document.cookie, cookieName), '1',
      'The cookie reaches /common/blank.html, so its path is "/"');
}, 'CONTROL a cookie set without a Path attribute here has a path of "/"');

// Control: the prefix is honoured when the Path attribute is there.
rootDocumentTest({
  name: 'CONTROL "__Host-" with an explicit Path of "/" is set',
  suffix: 'withpath',
  attributes: 'Secure; Path=/',
  stored: true,
});

// The cases under test. Each of these ends up with a path of "/" through the
// default path, without a Path attribute that says so.
rootDocumentTest({
  name: '"__Host-" without a Path attribute is not set',
  suffix: 'nopath',
  attributes: 'Secure',
  stored: false,
});

rootDocumentTest({
  name: '"__Host-" with an empty Path attribute is not set',
  suffix: 'emptypath',
  attributes: 'Secure; Path=',
  stored: false,
});

rootDocumentTest({
  name: '"__Host-" with a valueless Path attribute is not set',
  suffix: 'barepath',
  attributes: 'Secure; Path',
  stored: false,
});
