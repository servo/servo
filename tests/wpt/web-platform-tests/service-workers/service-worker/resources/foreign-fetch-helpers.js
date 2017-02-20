// Common helper functions for foreign fetch tests.

// Installs a service worker on a different origin. Both |worker| and |scope|
// are resolved relative to the /service-workers/service-worker/resources/
// directory on a remote origin.
function install_cross_origin_worker(
    t, worker, scope, origin = get_host_info().HTTPS_REMOTE_ORIGIN) {
  return with_iframe(origin + new URL('resources/install-worker-helper.html', location).pathname)
    .then(frame => new Promise((resolve, reject) => {
        frame.contentWindow.postMessage({worker: worker,
                                         options: {scope: scope}},
                                        '*');
        window.addEventListener('message', reply => {
            if (reply.source != frame.contentWindow) return;
            if (reply.data == 'success') resolve();
            else reject(reply.data);
          });
      }));
}

// Performs a fetch from a different origin. By default this performs a fetch
// from a window on that origin, but if |worker_type| is 'dedicated' or 'shared'
// the fetch is made from a worker on that origin instead.
// This uses a window rather than an iframe because an iframe might get blocked
// by mixed content checks.
function fetch_from_different_origin(origin, url, worker_type) {
  const win = open(origin + new URL('resources/foreign-fetch-helper-iframe.html', location).pathname);
  return new Promise(resolve => {
        self.addEventListener('message', e => {
            if (e.source != win) return;
            resolve();
          });
      })
    .then(() => new Promise((resolve) => {
        const channel = new MessageChannel();
        win.postMessage({url: url,
                         worker: worker_type},
                        '*', [channel.port1]);
        channel.port2.onmessage = reply => {
          win.close();
          resolve(reply.data);
        };
      }));
}
