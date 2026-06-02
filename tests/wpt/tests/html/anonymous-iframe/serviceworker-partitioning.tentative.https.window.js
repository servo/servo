// META: script=/common/utils.js

const sw_url = location.pathname.replace(/[^/]*$/, '') +
      "./resources/serviceworker-partitioning-helper.js";

promise_test(async t => {
  // Create 4 iframes (two normal and two credentialless ones) and register
  // a serviceworker with the same scope and url in all of them.
  //
  // Registering the same service worker again with the same url and
  // scope is a no-op. However, credentialless iframes get partitioned
  // service workers, so we should have a total of 2 service workers
  // at the end (one for the normal iframes and one for the credentialless
  // ones).
  let iframes = await Promise.all([
    { name: "normal", credentialless: false},
    { name: "normal_control", credentialless: false},
    { name: "credentialless", credentialless: true},
    { name: "credentialless_control", credentialless: true},
  ].map(async ({name, credentialless}) => {

    let iframe = await new Promise(resolve => {
      let iframe = document.createElement('iframe');
      iframe.onload = () => resolve(iframe);
      iframe.src = '/common/blank.html';
      if (credentialless) iframe.credentialless = true;
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
  assert_false(!!msgs[0]["credentialless"]);
  assert_false(!!msgs[0]["credentialless_control"]);

  // The "normal_control" iframe shares the same serviceworker as the "normal"
  // iframe.
  assert_true(!!msgs[1]["normal"]);
  assert_true(!!msgs[1]["normal_control"]);
  assert_false(!!msgs[1]["credentialless"]);
  assert_false(!!msgs[1]["credentialless_control"]);

  // The "credentialless" iframe serviceworker belongs to the "credentialless"
  // and the "credentialless_control" iframes.
  assert_false(!!msgs[2]["normal"]);
  assert_false(!!msgs[2]["normal_control"]);
  assert_true(!!msgs[2]["credentialless"]);
  assert_true(!!msgs[2]["credentialless_control"]);

  // The "credentialless_control" iframe shares the same serviceworker as the
  // "credentialless" iframe.
  assert_false(!!msgs[3]["normal"]);
  assert_false(!!msgs[3]["normal_control"]);
  assert_true(!!msgs[3]["credentialless"]);
  assert_true(!!msgs[3]["credentialless_control"]);

}, "credentialless iframes get partitioned service workers.");
