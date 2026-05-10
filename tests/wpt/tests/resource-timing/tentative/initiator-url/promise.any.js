// META: global=sharedworker,dedicatedworker
importScripts("/resources/testharness.js")
importScripts("/common/get-host-info.sub.js")
importScripts("../../resources/test-initiator.js")
importScripts("../../resources/loading-resource-lib.js")

const label = "initiator_url_promise_worker";
const resource = "/images/blue.png?"+label;
const hostInfo = get_host_info();
const expectedInitiatorUrl = hostInfo["ORIGIN"] +
  "/resource-timing/tentative/initiator-url/promise.any.worker.js";
Promise.resolve().then(function() {fetch_in_function(resource)});
initiator_url_test(resource, expectedInitiatorUrl, resource +
  " initiatorUrl from promise in worker thread", resource + " timeout");
