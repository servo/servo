// META: script=/common/utils.js

const sw_url = location.pathname.replace(/[^/]*$/, '') +
      "./resources/sharedworker-partitioning-helper.js";

promise_test(async t => {
  // Create 4 iframes (two normal and two credentialless ones) and create
  // a shared worker with the same url in all of them.
  //
  // Creating the same shared worker again with the same url is a
  // no-op. However, credentialless iframes get partitioned shared workers,
  // so we should have a total of 2 shared workers at the end (one for
  // the normal iframes and one for the credentialless ones).
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

    let sw = new iframe.contentWindow.SharedWorker(sw_url);
    return { iframe: iframe, name: name, sw: sw };
  }));

  // Ping each worker telling him which iframe it belongs to.
  await Promise.all(iframes.map(iframe => {
    iframe.sw.port.postMessage({ action: 'record', from: iframe.name});
    return new Promise(resolve => {
      iframe.sw.port.onmessage = event => {
        if (event.data.ack === iframe.name) resolve();
      }
    });
  }));

  // Ping each (iframe, sharedworker) asking for which messages it got.
  let msgs = await Promise.all(iframes.map(iframe => {
    iframe.sw.port.postMessage({ action: 'retrieve', from: iframe.name });
    return new Promise(resolve => {
      iframe.sw.port.onmessage = event => {
        if (event.data.ack === iframe.name) resolve(event.data.messages);
      }
    });
  }));

  // The "normal" iframe sharedworker belongs to the "normal" and the
  // "normal_control" iframes.
  assert_true(!!msgs[0]["normal"] &&
              !!msgs[0]["normal_control"] &&
              !msgs[0]["credentialless"] &&
              !msgs[0]["credentialless_control"],
              'The "normal" iframe\'s sharedworker should return ' +
              '{"normal": true, "normal_control": true}, ' +
              'but instead returned ' + JSON.stringify(msgs[0]));

  // The "normal_control" iframe shares the same sharedworker as the "normal"
  // iframe.
  assert_true(!!msgs[1]["normal"] &&
              !!msgs[1]["normal_control"] &&
              !msgs[1]["credentialless"] &&
              !msgs[1]["credentialless_control"],
              'The "normal_control" iframe\'s sharedworker should return ' +
              '{"normal": true, "normal_control": true}, ' +
              'but instead returned ' + JSON.stringify(msgs[1]));

  // The "credentialless" iframe sharedworker belongs to the "credentialless" and the
  // "credentialless_control" iframes.
  assert_true(!msgs[2]["normal"] &&
              !msgs[2]["normal_control"] &&
              !!msgs[2]["credentialless"] &&
              !!msgs[2]["credentialless_control"],
              'The "credentialless" iframe\'s sharedworker should return ' +
              '{"credentialless": true, "credentialless_control": true}, ' +
              'but instead returned ' + JSON.stringify(msgs[2]));

  // The "credentialless_control" iframe shares the same sharedworker as
  // the "credentialless" iframe.
  assert_true(!msgs[3]["normal"] &&
              !msgs[3]["normal_control"] &&
              !!msgs[3]["credentialless"] &&
              !!msgs[3]["credentialless_control"],
              'The "credentialless_control" iframe\'s sharedworker should return ' +
              '{"credentialless": true, "credentialless_control": true}, ' +
              'but instead returned ' + JSON.stringify(msgs[3]));

}, "credentialless iframes get partitioned shared workers.");
