self.GLOBAL = {
  isWindow: function() { return false; },
  isWorker: function() { return true; },
};
importScripts('/resources/testharness.js',
              '/resources/WebIDLParser.js',
              '/resources/idlharness.js');

promise_test(async t => {
  const urls = ['/interfaces/cookie-store.idl'];
  const [cookie_store] = await Promise.all(
    urls.map(url => fetch(url).then(response => response.text())));

  const idl_array = new IdlArray();

  idl_array.add_untested_idls(
    `[Global=ServiceWorker, Exposed=ServiceWorker]
     interface ServiceWorkerGlobalScope {};`);
  idl_array.add_untested_idls(
    `[Global=Window, Exposed=Window]
     interface Window {};`);

  idl_array.add_idls(cookie_store);

  idl_array.add_objects({
    CookieStore: [self.cookieStore],
  });
  idl_array.test();
}, 'Interface test');

done();
