'use strict';

importScripts('worker-testharness.js');
importScripts('/resources/WebIDLParser.js');
importScripts('/resources/idlharness.js');

promise_test(async (t) => {
  const srcs = ['dom', 'html', 'service-workers'];
  const [dom, html, serviceWorkerIdl] = await Promise.all(
    srcs.map(i => fetch(`/interfaces/${i}.idl`).then(r => r.text())));

  var idlArray = new IdlArray();
  idlArray.add_idls(serviceWorkerIdl, { only: [
    'ServiceWorkerGlobalScope',
    'Client',
    'WindowClient',
    'Clients',
    'ServiceWorker',
    'ServiceWorkerState',
    'ServiceWorkerUpdateViaCache',
    'ServiceWorkerRegistration',
    'EventTarget',
    'NavigationPreloadManager',
    'Cache',
    'CacheStorage',
  ]});
  idlArray.add_dependency_idls(dom);
  idlArray.add_dependency_idls(html);
  idlArray.add_objects({
    ServiceWorkerGlobalScope: ['self'],
    Clients: ['self.clients'],
    ServiceWorkerRegistration: ['self.registration'],
    CacheStorage: ['self.caches']
    // TODO: Test instances of Client and WindowClient, e.g.
    // Client: ['self.clientInstance'],
    // WindowClient: ['self.windowClientInstance']
  });
  return create_temporary_cache(t)
    .then(function(cache) {
        self.cacheInstance = cache;

        idlArray.add_objects({ Cache: ['self.cacheInstance'] });
        idlArray.test();
      });
}, 'test setup (cache creation)');

test(function() {
    var req = new Request('http://{{host}}/',
                          {method: 'POST',
                           headers: [['Content-Type', 'Text/Html']]});
    assert_equals(
      new ExtendableEvent('ExtendableEvent').type,
      'ExtendableEvent', 'Type of ExtendableEvent should be ExtendableEvent');
    assert_throws_js(TypeError, function() {
        new FetchEvent('FetchEvent');
    }, 'FetchEvent constructor with one argument throws');
    assert_throws_js(TypeError, function() {
        new FetchEvent('FetchEvent', {});
    }, 'FetchEvent constructor with empty init dict throws');
    assert_throws_js(TypeError, function() {
        new FetchEvent('FetchEvent', {request: null});
    }, 'FetchEvent constructor with null request member throws');
    assert_equals(
      new FetchEvent('FetchEvent', {request: req}).type,
      'FetchEvent', 'Type of FetchEvent should be FetchEvent');
    assert_equals(
      new FetchEvent('FetchEvent', {request: req}).cancelable,
      false, 'Default FetchEvent.cancelable should be false');
    assert_equals(
      new FetchEvent('FetchEvent', {request: req}).bubbles,
      false, 'Default FetchEvent.bubbles should be false');
    assert_equals(
      new FetchEvent('FetchEvent', {request: req}).clientId,
      '', 'Default FetchEvent.clientId should be the empty string');
    assert_equals(
      new FetchEvent('FetchEvent', {request: req, cancelable: false}).cancelable,
      false, 'FetchEvent.cancelable should be false');
    assert_equals(
      new FetchEvent('FetchEvent', {request: req, clientId : 'test-client-id'}).clientId, 'test-client-id',
      'FetchEvent.clientId with option {clientId : "test-client-id"} should be "test-client-id"');
    assert_equals(
      new FetchEvent('FetchEvent', {request : req}).request.url,
      'http://{{host}}/',
      'FetchEvent.request.url should return the value it was initialized to');
    assert_equals(
      new FetchEvent('FetchEvent', {request : req}).isReload,
      undefined,
      'FetchEvent.isReload should not exist');

  }, 'Event constructors');

test(() => {
    assert_false('XMLHttpRequest' in self);
  }, 'xhr is not exposed');

test(() => {
    assert_false('createObjectURL' in self.URL);
  }, 'URL.createObjectURL is not exposed')
