// META: global=sharedworker,dedicatedworker
importScripts("/resources/testharness.js")
importScripts("/common/get-host-info.sub.js")
importScripts("../../resources/test-initiator.js")
importScripts("../../resources/loading-resource-lib.js?direct_any")

const label = "initiator_url_direct_worker";
const imageResource = "/images/blue.png?"+label;
const hostInfo = get_host_info();
const workerUrl = hostInfo["ORIGIN"] +
  "/resource-timing/tentative/initiator-url/direct.any.worker.js";
fetch_in_function(imageResource);

// For a resource fetched by the worker, Initiator is the worker itself.
initiator_url_test(imageResource, workerUrl, imageResource +
  " initiatorUrl from worker thread", imageResource + " timeout");

// Initiator for a JS file imported by "importScripts()" is the worker itself.
const importScriptsUrl =
  "resource-timing/resources/loading-resource-lib.js?direct_any";
initiator_url_test(importScriptsUrl, workerUrl, importScriptsUrl +
  " initiatorUrl from worker thread", importScriptsUrl + " timeout");
