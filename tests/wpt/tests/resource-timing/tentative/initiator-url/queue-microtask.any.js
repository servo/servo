// META: global=sharedworker,dedicatedworker
importScripts("/resources/testharness.js")
importScripts("/common/get-host-info.sub.js")
importScripts("../../resources/test-initiator.js")
importScripts("../../resources/loading-resource-lib.js")

const label = "initiator_url_queuemicrotask_worker";
const resource = "/images/blue.png?"+label;
const expectedInitiatorUrl = getUrl(
    '/resource-timing/tentative/initiator-url/queue-microtask.any.worker.js');
queueMicrotask(function() {fetch_in_function(resource)});
initiator_url_test(resource, expectedInitiatorUrl, resource +
  " initiatorUrl from queueMicrotask() in worker thread", resource + " timeout");
