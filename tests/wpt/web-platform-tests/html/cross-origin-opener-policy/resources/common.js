const SAME_ORIGIN = {origin: get_host_info().HTTPS_ORIGIN, name: "SAME_ORIGIN"};
const SAME_SITE = {origin: get_host_info().HTTPS_REMOTE_ORIGIN, name: "SAME_SITE"};
const CROSS_ORIGIN = {origin: get_host_info().HTTPS_NOTSAMESITE_ORIGIN, name: "CROSS_ORIGIN"}

function url_test(t, url, channelName, hasOpener) {
  const bc = new BroadcastChannel(channelName);
  bc.onmessage = t.step_func_done(event => {
    const payload = event.data;
    assert_equals(payload.name, hasOpener ? channelName : "");
    assert_equals(payload.opener, hasOpener);
  });

  const w = window.open(url, channelName);

  // w will be closed by its postback iframe. When out of process,
  // window.close() does not work.
  t.add_cleanup(() => w.close());
}

function coop_coep_test(t, host, coop, coep, channelName, hasOpener) {
  url_test(t, `${host.origin}/html/cross-origin-opener-policy/resources/coop-coep.py?coop=${encodeURIComponent(coop)}&coep=${coep}&channel=${channelName}`, channelName, hasOpener);
}

function coop_test(t, host, coop, channelName, hasOpener) {
  coop_coep_test(t, host, coop, "", channelName, hasOpener);
}

function run_coop_tests(documentCOOPValueTitle, testArray) {
  for (const test of tests) {
    async_test(t => {
      coop_test(t, test[0], test[1],
                `${mainTest}_to_${test[0].name}_${test[1].replace(/ /g,"-")}`,
                test[2]);
    }, `${documentCOOPValueTitle} document opening popup to ${test[0].origin} with COOP: "${test[1]}"`);
  }
}
