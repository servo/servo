const ORIGINS = {
  'same-origin': get_host_info().HTTP_ORIGIN,
  'cross-origin': get_host_info().HTTP_REMOTE_ORIGIN,
  'cross-site': get_host_info().HTTP_NOTSAMESITE_ORIGIN,
}

function checkMeasureMemoryBreakdown(breakdown, options, required) {
  const allowed = new Set(options.allowed);
  assert_own_property(breakdown, 'bytes');
  assert_greater_than_equal(breakdown.bytes, 0);
  assert_own_property(breakdown, 'userAgentSpecificTypes');
  for (const userAgentSpecificType of breakdown.userAgentSpecificTypes) {
    assert_equals(typeof userAgentSpecificType, 'string');
  }
  assert_own_property(breakdown, 'attribution');
  for (const attribution of breakdown.attribution) {
    assert_equals(typeof attribution, 'string');
    assert_true(
        allowed.has(attribution),
        `${attribution} must be in ${JSON.stringify(options.allowed)}`);
    if (required.has(attribution)) {
      required.delete(attribution);
    }
  }
}

function checkMeasureMemory(result, options) {
    assert_own_property(result, 'bytes');
    assert_own_property(result, 'breakdown');
    const required = new Set(options.required);
    let bytes = 0;
    for (let breakdown of result.breakdown) {
      checkMeasureMemoryBreakdown(breakdown, options, required);
      bytes += breakdown.bytes;
    }
    assert_equals(bytes, result.bytes);
    assert_equals(required.size, 0, JSON.stringify(result.breakdown) +
        ' does not include ' + JSON.stringify(required.values()));
}

function url(params) {
  let origin = null;
  for (const key of Object.keys(ORIGINS)) {
    if (params.id.startsWith(key)) {
      origin = ORIGINS[key];
    }
  }
  const child = params.window_open ? 'window' : 'iframe';
  let file = `measure-memory/resources/${child}.sub.html`;
  if (params.redirect) {
    file = `measure-memory/resources/${child}.redirect.sub.html`;
  }
  let url = `${origin}/${file}?id=${params.id}`;
  if (params.redirect === 'server') {
    url = `${origin}/common/redirect.py?location=${encodeURIComponent(url)}`;
  }
  return url;
}

// A simple multiplexor of messages based on iframe ids.
let waitForMessage = (function () {
  class Inbox {
    constructor() {
      this.queue = [];
      this.resolve = null;
    }
    push(value) {
      if (this.resolve) {
        this.resolve(value);
        this.resolve = null;
      } else {
        this.queue.push(value);
      }
    }
    pop() {
      let promise = new Promise(resolve => this.resolve = resolve);
      if (this.queue.length > 0) {
        this.resolve(this.queue.shift());
        this.resolve = null;
      }
      return promise;
    }
  }
  const inbox = {};

  window.onmessage = function (message) {
    const id = message.data.id;
    const payload = message.data.payload;
    inbox[id] = inbox[id] || new Inbox();
    inbox[id].push(payload);
  }
  return function (id) {
    inbox[id] = inbox[id] || new Inbox();
    return inbox[id].pop();
  }
})();

// Constructs iframes based on their descriptoin.
async function build(children) {
  window.accessible_children = {iframes: {}, windows: {}};
  await Promise.all(children.map(buildChild));
  const result = window.accessible_children;
  delete window.accessible_children;
  return result;
}

async function buildChild(params) {
  let child = null;
  function target() {
    return params.window_open ? child : child.contentWindow;
  }
  if (params.window_open) {
    child = window.open(url(params));
  } else {
    child = document.createElement('iframe');
    child.src = url(params);
    child.id = params.id;
    document.body.appendChild(child);
  }
  const ready = await waitForMessage(params.id);
  target().postMessage({id: 'parent', payload: params.children}, '*');
  const done = await waitForMessage(params.id);
  let main = window;
  while (true) {
    if (main === main.parent) {
      if (!main.opener) {
        break;
      } else {
        main = main.opener;
      }
    } else {
      main = main.parent;
    }
  }
  try {
    main.accessible_children;
  } catch (e) {
    // Cross-origin iframe that cannot access the main frame.
    return;
  }
  if (params.window_open) {
    main.accessible_children.windows[params.id] = child;
  } else  {
    main.accessible_children.iframes[params.id] = child;
  }
}

function getId() {
  const params = new URLSearchParams(document.location.search);
  return params.get('id');
}

function getParent() {
  if (window.parent == window && window.opener) {
    return window.opener;
  }
  return window.parent;
}

// This function runs within an iframe.
// It gets the children descriptions from the parent and constructs them.
async function setupChild() {
  const id = getId();
  document.getElementById('title').textContent = id;
  getParent().postMessage({id : id, payload: 'ready'}, '*');
  const children = await waitForMessage('parent');
  if (children) {
    await build(children);
  }
  getParent().postMessage({id: id, payload: 'done'}, '*');
}

function sameOriginContexts(children) {
  const result = [];
  for (const [id, child] of Object.entries(children)) {
    if (id.includes('same-origin')) {
      result.push(child.contentWindow
          ? child.contentWindow.performance : child.performance);
    }
  }
  return result;
}

async function createWorker(bytes) {
  const worker = new Worker('resources/worker.js');
  let resolve_promise;
  const promise = new Promise(resolve => resolve_promise = resolve);
  worker.onmessage = function (message) {
    assert_equals(message.data, 'ready');
    resolve_promise();
  }
  worker.postMessage({bytes});
  return promise;
}