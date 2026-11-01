importScripts("/common/get-host-info.sub.js")
importScripts("/resource-timing/resources/loading-resource-lib.js")
importScripts('/resource-timing/resources/test-initiator.js')

const label = "initiator_url_message_handler_worker";
const resource = "/images/blue.png?"+label;
const expectedInitiatorUrl =
    getUrl('/resource-timing/resources/message-handler-in-worker.js');

self.onmessage = async function (event) {
  fetch_in_function(resource);
  const entry = await observe_entry_no_timeout(resource);
  postMessage({result: entry.initiatorUrl,
               expected:  expectedInitiatorUrl});
};
