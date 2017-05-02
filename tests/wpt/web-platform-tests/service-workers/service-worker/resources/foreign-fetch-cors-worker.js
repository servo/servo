importScripts('/common/get-host-info.sub.js');
var host_info = get_host_info();

self.addEventListener('install', function(event) {
    event.registerForeignFetch({scopes: [registration.scope], origins: ['*']});
  });

self.addEventListener('foreignfetch', function(event) {
    var response = JSON.parse(decodeURIComponent(location.search.substring(1)));
    var url = new URL(event.request.url);
    var params = JSON.parse(decodeURIComponent(url.search.substring(1)));
    var url_to_fetch = 'fetch-access-control.py?';
    if (params.cross_origin) {
      url_to_fetch =
          host_info.HTTPS_ORIGIN + new URL('./', location).pathname + url_to_fetch;
    }
    if (params.with_aceheaders)
      url_to_fetch += 'ACEHeaders=X-ServiceWorker-ServerHeader&';
    if (params.with_acaorigin)
      url_to_fetch += 'ACAOrigin=*';
    fetch_params = {};
    if (params.cross_origin && !params.with_acaorigin)
       fetch_params.mode = 'no-cors';
    event.respondWith(fetch(url_to_fetch, fetch_params)
      .then(r => {
          response.response = r;
          return response;
        }));
    return;
  });
