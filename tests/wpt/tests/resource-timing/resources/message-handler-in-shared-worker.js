importScripts("/common/get-host-info.sub.js")
importScripts("/resource-timing/resources/loading-resource-lib.js")

const label = "initiator_url_message_handler_shared_worker";
const resource = "/images/blue.png?"+label;
const hostInfo = get_host_info();
const expectedInitiatorUrl = hostInfo["ORIGIN"] +
  "/resource-timing/resources/message-handler-in-shared-worker.js";

const observe_entry_no_timeout = entryName => {
  const entry = new Promise(resolve => {
    new PerformanceObserver((entryList, observer) => {
      for (const entry of entryList.getEntries()) {
        if (entry.name.endsWith(entryName)) {
          resolve(entry);
          observer.disconnect();
          return;
        }
      }
    }).observe({"type": "resource", "buffered": true});
  });
  return entry;
};

self.onconnect = (e) => {
  const port = e.ports[0];
  port.onmessage = async function (event) {
    fetch_in_function(resource);
    const entry = await observe_entry_no_timeout(resource);
    port.postMessage({result: entry.initiatorUrl,
                       expected:  expectedInitiatorUrl});
  };
};
