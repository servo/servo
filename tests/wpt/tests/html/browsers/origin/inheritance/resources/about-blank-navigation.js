// Helpers for tests in which a document other than the parent or opener
// navigates a navigable to about:blank. The about:blank document then takes
// its origin, policy container, and base URL from that navigation's initiator:
// https://html.spec.whatwg.org/#navigate (initiatorOriginSnapshot)
//
// Requires /common/get-host-info.sub.js, /common/utils.js, and
// /common/dispatcher/dispatcher.js.

const hosts = get_host_info();

// The origin of the document that the test compares with the initiator, from
// the variant's first parameter. /common/subset-tests.js reads the second.
const variantOrigin = {
  "same-origin": hosts.ORIGIN,
  "same-site": hosts.REMOTE_ORIGIN,
  "cross-site": hosts.HTTP_NOTSAMESITE_ORIGIN,
}[location.search.substr(1).split("&")[0]];

// The initiator only allows images from initiatorImageOrigin, other documents
// only from otherImageOrigin.
const initiatorImageOrigin = hosts.ORIGIN;
const otherImageOrigin = hosts.OTHER_ORIGIN;

function imgSrc(origin) {
  return `|header(Content-Security-Policy,img-src ${origin})`;
}

function imageURLs() {
  return [initiatorImageOrigin, otherImageOrigin].map(origin => `${origin}/images/green.png?${token()}`);
}

// Documents send Origin-Agent-Cluster: ?0, so that document.domain works.
function executorURL(origin, uuid, headers = "") {
  return `${origin}/common/dispatcher/remote-executor.html?uuid=${uuid}&pipe=header(Origin-Agent-Cluster,%3F0)${headers}`;
}

async function openPopupContext(t, origin, headers) {
  const context = new RemoteContext(token());
  const popup = window.open(executorURL(origin, context.context_id, headers));
  t.add_cleanup(() => popup.close());
  await context.execute_script(() => {});
  return context;
}

// `beforeRemoval` runs before the iframe is removed.
async function createIframeContext(t, origin, headers, beforeRemoval = () => {}) {
  const context = new RemoteContext(token());
  const iframe = document.createElement("iframe");
  iframe.src = executorURL(origin, context.context_id, headers);
  document.body.append(iframe);
  t.add_cleanup(() => {
    beforeRemoval(iframe);
    iframe.remove();
  });
  await context.execute_script(() => {});
  return context;
}

// Appends an iframe to the document of `context` and waits for it to load.
function addIframe(context, url, sandbox = null) {
  return context.execute_script(async (url, sandbox) => {
    const iframe = document.createElement("iframe");
    if (sandbox !== null)
      iframe.setAttribute("sandbox", sandbox);
    iframe.src = url;
    document.body.append(iframe);
    await new Promise(resolve => iframe.onload = resolve);
  }, [url, sandbox]);
}

async function addIframeContext(context, origin, headers) {
  const child = new RemoteContext(token());
  await addIframe(context, executorURL(origin, child.context_id, headers));
  return child;
}

// Returns a function that waits until the iframe at `index` in the document of
// `context` has loaded its next document. The next navigation comes from a later
// task, so it doesn't replace the session history entry.
async function expectIframeLoad(context, index = 0) {
  await context.execute_script(index => {
    const iframe = document.querySelectorAll("iframe")[index];
    window.iframeLoaded = new Promise(resolve => {
      iframe.addEventListener("load", resolve, { once: true });
    });
  }, [index]);
  return () => context.execute_script(async () => {
    await window.iframeLoaded;
  });
}

// `path` names a window starting from the global object of `context`, such as
// ["frames", 0, 0] or ["opener"]. Resolves with what `context` can see of that
// window's document, or with the name of the exception that accessing it
// throws.
function inspect(context, path) {
  return context.execute_script(path => {
    try {
      const w = path.reduce((w, step) => w[step], self);
      return { url: w.document.URL, origin: w.origin, baseURI: w.document.baseURI };
    } catch (e) {
      return { error: e.name };
    }
  }, [path]);
}

// Appends an image for each URL to the document of the window at `path`, and
// resolves with "loaded" or "blocked" for each, or with the name of the
// exception that accessing the document throws.
function loadImages(context, path, urls) {
  return context.execute_script(async (path, urls) => {
    let doc;
    try {
      doc = path.reduce((w, step) => w[step], self).document;
    } catch (e) {
      return e.name;
    }
    const results = await Promise.all(urls.map(url => new Promise(resolve => {
      const img = doc.createElement("img");
      img.onload = () => resolve("loaded");
      img.onerror = () => resolve("blocked");
      img.src = url;
      doc.body.append(img);
    })));
    return results.join(" ");
  }, [path, urls]);
}

function setDocumentDomain(context) {
  return context.execute_script(() => {
    document.domain = document.domain;
  });
}

function setBaseURL(context, href) {
  return context.execute_script(href => {
    const base = document.createElement("base");
    base.href = href;
    document.head.append(base);
  }, [href]);
}

function assert_can_script(result, origin, description) {
  assert_equals(result.error, undefined, `${description} can script the document`);
  assert_equals(result.url, "about:blank", `${description} sees the URL`);
  assert_equals(result.origin, origin, `${description} sees the origin`);
}

function assert_cannot_script(result, description) {
  assert_equals(result.error, "SecurityError", `${description} can't script the document`);
}
