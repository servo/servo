const SAME_ORIGIN = {origin: get_host_info().HTTP_ORIGIN, name: "SAME_ORIGIN"};
const SAME_SITE = {origin: get_host_info().HTTP_REMOTE_ORIGIN, name: "SAME_SITE"};
const CROSS_ORIGIN = {origin: get_host_info().HTTP_NOTSAMESITE_ORIGIN, name: "CROSS_ORIGIN"}

function coop_test(t, host, coop, channelName, hasOpener) {
  let bc = new BroadcastChannel(channelName);
  bc.onmessage = t.step_func_done((event) => {
    let payload = event.data;
    assert_equals(payload.name, hasOpener ? channelName : "");
    assert_equals(payload.opener, hasOpener);
  });

  let w = window.open(`${host.origin}/html/cross-origin-opener/resources/coop_window.py?path=window.sub.html&coop=${escape(coop)}&channel=${channelName}`, channelName);

  // w will be closed by its postback iframe. When out of process,
  // window.close() does not work.
  t.add_cleanup(() => w.close());
}

function run_coop_tests(mainTest, testArray) {
  for (let test of tests) {
   async_test(t => {
    coop_test(t, test[0], test[1],
              `${mainTest}_to_${test[0].name}_${test[1].replace(/ /g,"-")}`,
              test[2]);
  }, `${mainTest} document opening popup to ${test[0].origin} with COOP: "${test[1]}"`);
 }
}
