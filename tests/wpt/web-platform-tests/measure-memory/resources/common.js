const ORIGINS = {
  'same-origin': get_host_info().HTTPS_ORIGIN,
  'cross-origin': get_host_info().HTTPS_REMOTE_ORIGIN,
  'cross-site': get_host_info().HTTPS_NOTSAMESITE_ORIGIN,
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
    url = (`${origin}/measure-memory/resources/redirect.py?` +
           `location=${encodeURIComponent(url)}`);
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

function getMainWindow() {
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
  return main;
}

function isSameOrigin(other) {
  try {
    other.descendants;
  } catch (e) {
    // Cross-origin iframe that cannot access the main frame.
    return false;
  }
  return !!other.descendants;
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

// Constructs iframes based on their descriptoin.
async function build(children) {
  window.descendants = {iframes: {}, windows: {}};
  await Promise.all(children.map(buildChild));
  const result = window.descendants;
  return result;
}

async function buildChild(params) {
  let child = null;
  function target() {
    return params.window_open ? child : child.contentWindow;
  }
  if (params.window_open) {
    child = window.open(url(params));
    if (!params.id.startsWith('same-origin')) {
      // Cross-origin windows gets their own browsing context groups with COOP.
      // The postMessage calls before would not work for them, so we do not
      // wait for them to load.
      return;
    }
  } else {
    child = document.createElement('iframe');
    child.src = url(params);
    child.id = params.id;
    document.body.appendChild(child);
  }
  const ready = await waitForMessage(params.id);
  target().postMessage({id: 'parent', payload: params.children}, '*');
  const done = await waitForMessage(params.id);
  if (!params.window_open) {
    const main = getMainWindow();
    if (isSameOrigin(main)) {
      main.descendants.iframes[params.id] = child;
    }
  }
}

// This function runs within an iframe.
// It gets the children descriptions from the parent and constructs them.
async function setupChild() {
  const id = getId();
  const main = getMainWindow();
  if (isSameOrigin(main)) {
    main.descendants.windows[id] = window;
  }
  document.getElementById('title').textContent = id;
  getParent().postMessage({id : id, payload: 'ready'}, '*');
  const children = await waitForMessage('parent');
  if (children) {
    await Promise.all(children.map(buildChild));
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
    resolve_promise(message.data);
  }
  worker.postMessage({bytes});
  return promise;
}