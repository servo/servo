var idls = {};
idls.untested = `
[Exposed=Worker]
interface WorkerGlobalScope {};

[TreatNonObjectAsNull]
callback EventHandlerNonNull = any (Event event);
typedef EventHandlerNonNull? EventHandler;

[NoInterfaceObject, Exposed=(Window,Worker)]
interface AbstractWorker {
  attribute EventHandler onerror;
};
`;
idls.tested = `
[Global=(Worker,ServiceWorker), Exposed=ServiceWorker]
interface ServiceWorkerGlobalScope : WorkerGlobalScope {
  [SameObject] readonly attribute Clients clients;
  [SameObject] readonly attribute ServiceWorkerRegistration registration;

  [NewObject] Promise<void> skipWaiting();

  attribute EventHandler oninstall;
  attribute EventHandler onactivate;
  attribute EventHandler onfetch;

  // event
  attribute EventHandler onmessage; // event.source of the message events is Client object
  attribute EventHandler onmessageerror;
};

[Exposed=ServiceWorker]
interface Client {
  readonly attribute USVString url;
  readonly attribute DOMString id;
  readonly attribute ClientType type;
  readonly attribute boolean reserved;
  void postMessage(any message, optional sequence<object> transfer = []);
};

[Exposed=ServiceWorker]
interface WindowClient : Client {
  readonly attribute VisibilityState visibilityState;
  readonly attribute boolean focused;
  [SameObject] readonly attribute FrozenArray<USVString> ancestorOrigins;
  [NewObject] Promise<WindowClient> focus();
  [NewObject] Promise<WindowClient> navigate(USVString url);
};

[Exposed=ServiceWorker]
interface Clients {
  // The objects returned will be new instances every time
  [NewObject] Promise<any> get(DOMString id);
  [NewObject] Promise<sequence<Client>> matchAll(optional ClientQueryOptions options);
  [NewObject] Promise<WindowClient?> openWindow(USVString url);
  [NewObject] Promise<void> claim();
};

[SecureContext, Exposed=(Window,Worker)]
interface ServiceWorker : EventTarget {
  readonly attribute USVString scriptURL;
  readonly attribute ServiceWorkerState state;
  void postMessage(any message, optional sequence<object> transfer = []);

  // event
  attribute EventHandler onstatechange;
};
ServiceWorker implements AbstractWorker;

enum ServiceWorkerState {
  "installing",
  "installed",
  "activating",
  "activated",
  "redundant"
};

enum ServiceWorkerUpdateViaCache {
    "imports",
    "all",
    "none"
};

[SecureContext, Exposed=(Window,Worker)]
interface ServiceWorkerRegistration : EventTarget {
  readonly attribute ServiceWorker? installing;
  readonly attribute ServiceWorker? waiting;
  readonly attribute ServiceWorker? active;
  [SameObject] readonly attribute NavigationPreloadManager navigationPreload;

  readonly attribute USVString scope;
  readonly attribute ServiceWorkerUpdateViaCache updateViaCache;

  [NewObject] Promise<void> update();
  [NewObject] Promise<boolean> unregister();

  // event
  attribute EventHandler onupdatefound;
};

[Constructor(), Exposed=(Window,Worker)]
interface EventTarget {
  void addEventListener(DOMString type, EventListener? callback, optional (AddEventListenerOptions or boolean) options);
  void removeEventListener(DOMString type, EventListener? callback, optional (EventListenerOptions or boolean) options);
  boolean dispatchEvent(Event event);
};

[SecureContext, Exposed=(Window,Worker)]
interface NavigationPreloadManager {
  Promise<void> enable();
  Promise<void> disable();
  Promise<void> setHeaderValue(ByteString value);
  Promise<NavigationPreloadState> getState();
};

[SecureContext, Exposed=(Window,Worker)]
interface Cache {
  [NewObject] Promise<any> match(RequestInfo request, optional CacheQueryOptions options);
  [NewObject] Promise<sequence<Response>> matchAll(optional RequestInfo request, optional CacheQueryOptions options);
  [NewObject] Promise<void> add(RequestInfo request);
  [NewObject] Promise<void> addAll(sequence<RequestInfo> requests);
  [NewObject] Promise<void> put(RequestInfo request, Response response);
  [NewObject] Promise<boolean> delete(RequestInfo request, optional CacheQueryOptions options);
  [NewObject] Promise<sequence<Request>> keys(optional RequestInfo request, optional CacheQueryOptions options);
};

[SecureContext, Exposed=(Window,Worker)]
interface CacheStorage {
  [NewObject] Promise<any> match(RequestInfo request, optional CacheQueryOptions options);
  [NewObject] Promise<boolean> has(DOMString cacheName);
  [NewObject] Promise<Cache> open(DOMString cacheName);
  [NewObject] Promise<boolean> delete(DOMString cacheName);
  [NewObject] Promise<sequence<DOMString>> keys();
};
`;
