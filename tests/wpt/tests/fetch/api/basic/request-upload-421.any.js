// META: global=window,worker
// META: script=../resources/utils.js
// META: script=/common/utils.js
// META: script=/common/get-host-info.sub.js

promise_test(async (test) => {
  const resp = await fetch(
    "/fetch/connection-pool/resources/network-partition-key.py?"
    + `status=421&uuid=${token()}&partition_id=${get_host_info().ORIGIN}`
    + `&dispatch=check_partition&addcounter=true`,
    {method: "POST", body: "foobar"});
  assert_equals(resp.status, 421);
  const text = await resp.text();
  assert_equals(text, "ok. Request was sent 2 times. 2 connections were created.");
}, "Fetch with POST with text body on 421 response should be retried once on new connection.");
