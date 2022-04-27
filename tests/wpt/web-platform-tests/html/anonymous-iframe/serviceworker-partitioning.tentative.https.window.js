// META: script=/common/utils.js

const sw_url = location.pathname.replace(/[^/]*$/, '') +
      "./resources/serviceworker-partitioning-helper.js";

promise_test(async t => {
  // Create 4 iframes (two normal and two anonymous ones) and register
  // a serviceworker with the same scope and url in all of them.
  //
  // Registering the same service worker again with the same url and
  // scope is a no-op. However, anonymous iframes get partitioned
  // service workers, so we should have a total of 2 service workers
  // at the end (one for the normal iframes and one for the anonymous
  // ones).
  let iframes = await Promise.all([
    { name: "normal", anonymous: false},
    { name: "normal_control", anonymous: false},
    { name: "anonymous", anonymous: true},
    { name: "anonymous_control", anonymous: true},
  ].map(async ({name, anonymous}) => {

    let iframe = await new Promise(resolve => {
      let iframe = document.createElement('iframe');
      iframe.onload = () => resolve(iframe);
      iframe.src = '/common/blank.html';
      if (anonymous) iframe.anonymous = true;
      document.body.append(iframe);
    });

    let sw = await new Promise(resolve => {
      iframe.contentWindow.navigator.serviceWorker.register(sw_url)
        .then(r => {
          add_completion_callback(_ => r.unregister());
          resolve(r.active || r.installing || r.waiting);
        });
    });
    return { iframe: iframe, name: name, sw: sw };
  }));

  // Setup a MessageChannel for each pair (iframe, serviceworker).
  // Ping each serviceworker telling him which iframe it belongs to.
  iframes.forEach((iframe, i) => {
    iframe.channel = new MessageChannel();
    iframe.sw.postMessage({ from: iframe.name }, [iframe.channel.port2]);
  });

  let msg_promises = iframes.map(iframe => new Promise(resolve => {
    iframe.channel.port1.onmessage = event => resolve(event.data);
  }));

  // Ping each (iframe, serviceworker) asking for which messages it got.
  iframes.map(iframe => iframe.sw.postMessage({ check: iframe.name }));

  // Collect all replies.
  let msgs = await Promise.all(msg_promises);

  // The "normal" iframe serviceworker belongs to the "normal" and the
  // "normal_control" iframes.
  assert_true(!!msgs[0]["normal"]);
  assert_true(!!msgs[0]["normal_control"]);
  assert_false(!!msgs[0]["anonymous"]);
  assert_false(!!msgs[0]["anonymous_control"]);

  // The "normal_control" iframe shares the same serviceworker as the "normal"
  // iframe.
  assert_true(!!msgs[1]["normal"]);
  assert_true(!!msgs[1]["normal_control"]);
  assert_false(!!msgs[1]["anonymous"]);
  assert_false(!!msgs[1]["anonymous_control"]);

  // The "anonymous" iframe serviceworker belongs to the "anonymous" and the
  // "anonymous_control" iframes.
  assert_false(!!msgs[2]["normal"]);
  assert_false(!!msgs[2]["normal_control"]);
  assert_true(!!msgs[2]["anonymous"]);
  assert_true(!!msgs[2]["anonymous_control"]);

  // The "anonymous_control" iframe shares the same serviceworker as
  // the "anonymous" iframe.
  assert_false(!!msgs[3]["normal"]);
  assert_false(!!msgs[3]["normal_control"]);
  assert_true(!!msgs[3]["anonymous"]);
  assert_true(!!msgs[3]["anonymous_control"]);

}, "Anonymous iframes get partitioned service workers.");
