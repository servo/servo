// Helper functions used in web-bundle tests.

function addElementAndWaitForLoad(element) {
  return new Promise((resolve, reject) => {
    element.onload = () => resolve(element);
    element.onerror = () => reject(element);
    document.body.appendChild(element);
  });
}

function addElementAndWaitForError(element) {
  return new Promise((resolve, reject) => {
    element.onload = () => reject(element);
    element.onerror = () => resolve(element);
    document.body.appendChild(element);
  });
}

// Evaluates |code| in |iframe|. The following message event handler must be
// registered on the iframe page:
//   window.addEventListener(
//       'message',
//       (e) => { e.source.postMessage(eval(e.data), e.origin); });
function evalInIframe(iframe, code) {
  const message_promise = new Promise((resolve) => {
      window.addEventListener(
          'message',
          (e) => { resolve(e.data); },
          { once : true });
    });
  iframe.contentWindow.postMessage(code,'*');
  return message_promise;
}

function fetchAndWaitForReject(url) {
  return new Promise((resolve, reject) => {
    fetch(url)
      .then(() => {
        reject();
      })
      .catch(() => {
        resolve();
      });
  });
}

function isValidCrossOriginAttribute(crossorigin) {
  if (crossorigin === undefined)
    return true;
  if ((typeof crossorigin) != 'string')
    return false;
  const lower_crossorigin = crossorigin.toLowerCase();
  return (lower_crossorigin === 'anonymous') ||
         (lower_crossorigin  === 'use-credentials');
}

function addScriptAndWaitForError(url) {
  return new Promise((resolve, reject) => {
    const script = document.createElement("script");
    script.src = url;
    script.onload = reject;
    script.onerror = resolve;
    document.body.appendChild(script);
  });
}

function addScriptAndWaitForExecution(url) {
  return new Promise((resolve, reject) => {
    window.scriptLoaded = (val) => {
      window.scriptLoaded = undefined;
      resolve(val);
    };
    const script = document.createElement("script");
    script.src = url;
    script.onerror = reject;
    document.body.appendChild(script);
  });
}

function createWebBundleElement(url, resources, options) {
  const script = document.createElement("script");
  script.type = "webbundle";
  const json_rule  = {"source": url, "resources": resources};
  if (options && options.scopes) {
    json_rule.scopes = options.scopes;
  }
  if (options && options.credentials) {
    json_rule.credentials = options.credentials;
  }
  script.textContent = JSON.stringify(json_rule);
  return script;
}

function addWebBundleElementAndWaitForLoad(url, resources, options) {
  const element = createWebBundleElement(url, resources, options);
  return addElementAndWaitForLoad(element);
}

function addWebBundleElementAndWaitForError(url, resources, options) {
  const element = createWebBundleElement(url, resources, options);
  return addElementAndWaitForError(element);
}

// This function creates a new WebBundle element that has a rule
// constructed in accordance with a JSON object |new_rule|:
// 1. Copy over WebBundle rules from an existing element that are
// not present in |new_rule|: source, resources, scopes and credentials.
// 2. Then create a new WebBundle element from |new_rule| (that now
// has full information required after 1.) and return it.
function createNewWebBundleElementWithUpdatedRule(element, new_rule) {
  const rule = JSON.parse(element.textContent);
  if (rule.resources && !new_rule.resources)
    new_rule.resources = rule.resources;
  if (rule.scopes && !new_rule.scopes)
    new_rule.scopes = rule.scopes;
  if (rule.credentials && !new_rule.credentials)
    new_rule.credentials = rule.credentials;
  if (!new_rule.url)
    new_rule.url = rule.source;

  return createWebBundleElement(new_rule.url, new_rule.resources, new_rule);
}
