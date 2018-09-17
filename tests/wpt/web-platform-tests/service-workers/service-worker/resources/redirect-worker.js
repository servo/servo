// We store an empty response for each fetch event request we see
// in this Cache object so we can get the list of urls in the
// message event.
var cacheName = 'urls-' + self.registration.scope;

var waitUntilPromiseList = [];

async function getRequestInfos(event) {
  // Wait for fetch events to finish.
  await Promise.all(waitUntilPromiseList);
  waitUntilPromiseList = [];

  // Generate the message.
  const cache = await caches.open(cacheName);
  const requestList = await cache.keys();
  const requestInfos = [];
  for (let i = 0; i < requestList.length; i++) {
    requestInfos[i] = {
      url: requestList[i].url,
    };
  }
  await caches.delete(cacheName);

  event.data.port.postMessage({requestInfos});
}

self.addEventListener('message', async function(event) {
  if (event.data.command == 'getRequestInfos') {
    event.waitUntil(getRequestInfos(event));
    return;
  }

  // TODO(falken): Add a getClientInfos command to test Clients API.
});

function get_query_params(url) {
  var search = (new URL(url)).search;
  if (!search) {
    return {};
  }
  var ret = {};
  var params = search.substring(1).split('&');
  params.forEach(function(param) {
      var element = param.split('=');
      ret[decodeURIComponent(element[0])] = decodeURIComponent(element[1]);
    });
  return ret;
}

self.addEventListener('fetch', function(event) {
    var waitUntilPromise = caches.open(cacheName).then(function(cache) {
      return cache.put(event.request, new Response());
    });
    event.waitUntil(waitUntilPromise);

    var params = get_query_params(event.request.url);
    if (!params['sw']) {
      // To avoid races, add the waitUntil() promise to our global list.
      // If we get a message event before we finish here, it will wait
      // these promises to complete before proceeding to read from the
      // cache.
      waitUntilPromiseList.push(waitUntilPromise);
      return;
    }

    event.respondWith(waitUntilPromise.then(function() {
      if (params['sw'] == 'gen') {
        return Response.redirect(params['url']);
      } else if (params['sw'] == 'fetch') {
        return fetch(event.request);
      } else if (params['sw'] == 'fetch-url') {
        return fetch(params['url']);
      } else if (params['sw'] == 'follow') {
        return fetch(new Request(event.request.url, {redirect: 'follow'}));
      } else if (params['sw'] == 'manual') {
        return fetch(new Request(event.request.url, {redirect: 'manual'}));
      } else if (params['sw'] == 'manualThroughCache') {
        var url = event.request.url;
        var cache;
        return caches.delete(url)
          .then(function() { return self.caches.open(url); })
          .then(function(c) {
            cache = c;
            return fetch(new Request(url, {redirect: 'manual'}));
          })
          .then(function(res) { return cache.put(event.request, res); })
          .then(function() { return cache.match(url); });
      }

      // unexpected... trigger an interception failure
    }));
  });
