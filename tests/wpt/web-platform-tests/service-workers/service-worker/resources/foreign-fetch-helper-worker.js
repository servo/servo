importScripts('/common/get-host-info.sub.js');
const host_info = get_host_info();

self.onfetch = e => {
  const remote_url = host_info.HTTPS_REMOTE_ORIGIN +
                     new URL('./', location).pathname + 'simple.txt?basic_sw';
  e.respondWith(fetch(remote_url));
};
