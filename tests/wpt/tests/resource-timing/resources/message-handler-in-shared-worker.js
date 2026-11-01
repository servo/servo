importScripts("/common/get-host-info.sub.js")
importScripts("/resource-timing/resources/loading-resource-lib.js")
importScripts('/resource-timing/resources/test-initiator.js')

const label = "initiator_url_message_handler_shared_worker";
const resource = "/images/blue.png?"+label;
const expectedInitiatorUrl =
    getUrl('/resource-timing/resources/message-handler-in-shared-worker.js');

self.onconnect = (e) => {
  const port = e.ports[0];
  port.onmessage = async function (event) {
    fetch_in_function(resource);
    const entry = await observe_entry_no_timeout(resource);
    port.postMessage({result: entry.initiatorUrl,
                       expected:  expectedInitiatorUrl});
  };
};
