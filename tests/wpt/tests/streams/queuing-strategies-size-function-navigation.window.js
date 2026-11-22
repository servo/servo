// META: script=/common/get-host-info.sub.js
// META: timeout=long

'use strict';

const {
  ORIGIN,
  ORIGINAL_HOST,
  PORT2,
  REMOTE_ORIGIN,
  HTTP_NOTSAMESITE_ORIGIN,
  HTTPS_NOTSAMESITE_ORIGIN,
} = get_host_info();

const SAME_HOST_DIFFERENT_PORT_ORIGIN =
    `${location.protocol}//${ORIGINAL_HOST}:${PORT2}`;
const NOTSAMESITE_ORIGIN = location.protocol === 'https:' ?
    HTTPS_NOTSAMESITE_ORIGIN :
    HTTP_NOTSAMESITE_ORIGIN;

const FRAME_PATH = '/streams/resources/queuing-strategy-size-frame.html';

const NAVIGATION_TARGETS = [
  {id: 'same-origin', label: 'same-origin', origin: ORIGIN},
  {
    id: 'same-host-port',
    label: 'same-host different-port',
    origin: SAME_HOST_DIFFERENT_PORT_ORIGIN,
  },
  {id: 'subdomain', label: 'subdomain', origin: REMOTE_ORIGIN},
  {id: 'cross-site', label: 'cross-site', origin: NOTSAMESITE_ORIGIN},
];

function waitForTokenMessage(t, token, expectedStatus) {
  return new Promise(resolve => {
    function listener(event) {
      if (event.data && event.data.token === token &&
          event.data.status === expectedStatus) {
        window.removeEventListener('message', listener);
        resolve(event.data);
      }
    }
    window.addEventListener('message', listener);
    t.add_cleanup(() => window.removeEventListener('message', listener));
  });
}

async function createInitialFrame(t, tokenPrefix) {
  const iframe = document.createElement('iframe');
  document.body.appendChild(iframe);
  t.add_cleanup(() => iframe.remove());

  const initToken = `${tokenPrefix}-init`;
  const loaded = waitForTokenMessage(t, initToken, 'loaded');
  iframe.src = `${ORIGIN}${FRAME_PATH}?mode=idle&token=${initToken}`;
  await loaded;
  return iframe;
}

for (const type of ['ByteLengthQueuingStrategy', 'CountQueuingStrategy']) {
  const expectedSize = chunk =>
      type === 'ByteLengthQueuingStrategy' ? chunk.byteLength : 1;

  for (const target of NAVIGATION_TARGETS) {
    promise_test(async t => {
      const tokenBase = `${type}-${target.id}-cached`;
      const iframe = await createInitialFrame(t, tokenBase);

      const initialFunctionPrototype = iframe.contentWindow.Function.prototype;
      const initialFunctionConstructor = iframe.contentWindow.Function;
      const firstQs = new iframe.contentWindow[type]({highWaterMark: 1});
      const secondQs = new iframe.contentWindow[type]({highWaterMark: 1});

      const preNavSize = firstQs.size;
      assert_equals(
          firstQs.size, preNavSize,
          'repeated size reads in an attached realm should return the same function');
      assert_equals(
          secondQs.size, preNavSize,
          'instances in the same realm should share the cached size function');
      assert_equals(
          Object.getPrototypeOf(preNavSize), initialFunctionPrototype,
          'pre-navigation size function prototype must match its creation realm');
      assert_equals(
          preNavSize.constructor, initialFunctionConstructor,
          'pre-navigation size function constructor must match its creation realm');

      const navToken = `${tokenBase}-nav`;
      const cached = waitForTokenMessage(t, navToken, 'cached');
      iframe.src = `${target.origin}${FRAME_PATH}?type=${
          type}&mode=cache&token=${navToken}`;
      const navInfo = await cached;
      assert_true(
          navInfo.sameFunctionIdentity,
          'navigated realm should cache its own size function across instances');
      assert_true(
          navInfo.hasRealmFunctionPrototype,
          'navigated realm size function must use its own Function.prototype');
      assert_true(
          navInfo.hasRealmFunctionConstructor,
          'navigated realm size function must use its own Function constructor');

      // Do not assert `postNavSize === preNavSize` or repeated identity on
      // the detached realm: implementations may drop per-context caches on
      // detachment to avoid retaining JS objects from detached realms, as
      // long as the returned function belongs to the strategy's relevant
      // realm.
      const postNavSize = firstQs.size;
      assert_equals(typeof postNavSize, 'function');
      assert_equals(postNavSize.name, 'size');
      assert_equals(
          Object.getPrototypeOf(postNavSize), initialFunctionPrototype,
          'size function prototype must match the strategy relevant realm');
      assert_equals(
          postNavSize.constructor, initialFunctionConstructor,
          'size function constructor must match the strategy relevant realm');
      assert_equals(postNavSize({byteLength: 17}),
                    expectedSize({byteLength: 17}));
    }, `${type} size getter preserves relevant realm across ${target.label} navigation after caching`);

    promise_test(
        async t => {
          const tokenBase = `${type}-${target.id}-lazy`;
          const iframe = await createInitialFrame(t, tokenBase);

          const initialFunctionPrototype =
              iframe.contentWindow.Function.prototype;
          const initialFunctionConstructor = iframe.contentWindow.Function;
          const firstQs = new iframe.contentWindow[type]({highWaterMark: 1});

          const navToken = `${tokenBase}-nav`;
          const cached = waitForTokenMessage(t, navToken, 'cached');
          iframe.src = `${target.origin}${FRAME_PATH}?type=${
              type}&mode=cache&token=${navToken}`;
          const navInfo = await cached;
          assert_true(
              navInfo.sameFunctionIdentity,
              'navigated realm should cache its own size function across instances');
          assert_true(
              navInfo.hasRealmFunctionPrototype,
              'navigated realm size function must use its own Function.prototype');
          assert_true(
              navInfo.hasRealmFunctionConstructor,
              'navigated realm size function must use its own Function constructor');

          const sizeFn = firstQs.size;
          assert_equals(typeof sizeFn, 'function');
          assert_equals(sizeFn.name, 'size');
          assert_equals(
              Object.getPrototypeOf(sizeFn), initialFunctionPrototype,
              'size function prototype must match the strategy relevant realm');
          assert_equals(
              sizeFn.constructor, initialFunctionConstructor,
              'size function constructor must match the strategy relevant realm');
          assert_equals(sizeFn({byteLength: 17}),
                        expectedSize({byteLength: 17}));
        },
        `${type} size getter uses relevant realm when first accessed after ${
            target.label} navigation`);

    promise_test(
        async t => {
          const tokenBase = `${type}-${target.id}-order`;
          const iframe = await createInitialFrame(t, tokenBase);

          const initialFunctionPrototype =
              iframe.contentWindow.Function.prototype;
          const firstQs = new iframe.contentWindow[type]({highWaterMark: 1});

          const navToken = `${tokenBase}-nav`;
          const ready = waitForTokenMessage(t, navToken, 'ready');
          iframe.src = `${target.origin}${FRAME_PATH}?type=${
              type}&mode=defer&token=${navToken}`;
          await ready;

          const preNavRealmSize = firstQs.size;
          assert_equals(
              Object.getPrototypeOf(preNavRealmSize), initialFunctionPrototype,
              'size function prototype must match the strategy relevant realm');

          const evaluated = waitForTokenMessage(t, navToken, 'evaluated');
          iframe.contentWindow.postMessage(
              {token: navToken, command: 'evaluate'}, '*');
          const reply = await evaluated;

          assert_true(
              reply.sameFunctionIdentity,
              'navigated realm should cache a single size function across instances');
          assert_true(
              reply.hasRealmFunctionPrototype,
              'navigated realm size function must use its own Function.prototype');
          assert_true(
              reply.hasRealmFunctionConstructor,
              'navigated realm size function must use its own Function constructor');
          assert_equals(reply.result, expectedSize({byteLength: 42}));
        },
        `${type} size getter in navigated realm is independent of pre-navigation instance (${
            target.label})`);
  }
}
