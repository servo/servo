// Helper functions used in web-bundle tests.

function addElementAndWaitForLoad(element) {
  return new Promise((resolve, reject) => {
    element.onload = resolve;
    element.onerror = reject;
    document.body.appendChild(element);
  });
}

function addElementAndWaitForError(element) {
  return new Promise((resolve, reject) => {
    element.onload = reject;
    element.onerror = resolve;
    document.body.appendChild(element);
  });
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

function addLinkAndWaitForLoad(url, resources, crossorigin) {
  return new Promise((resolve, reject) => {
    if (!isValidCrossOriginAttribute(crossorigin)) {
      reject('invalid crossorigin attribute: ' + crossorigin);
      return;
    }
    const link = document.createElement("link");
    link.rel = "webbundle";
    link.href = url;
    if (crossorigin) {
      link.crossOrigin = crossorigin;
    }
    for (const resource of resources) {
      link.resources.add(resource);
    }
    link.onload = () => resolve(link);
    link.onerror = () => reject(link);
    document.body.appendChild(link);
  });
}

function addLinkAndWaitForError(url, resources, crossorigin) {
  return new Promise((resolve, reject) => {
    if (!isValidCrossOriginAttribute(crossorigin)) {
      reject('invalid crossorigin attribute: ' + crossorigin);
      return;
    }
    const link = document.createElement("link");
    link.rel = "webbundle";
    link.href = url;
    if (crossorigin) {
      link.crossOrigin = crossorigin;
    }
    for (const resource of resources) {
      link.resources.add(resource);
    }
    link.onload = () => reject(link);
    link.onerror = () => resolve(link);
    document.body.appendChild(link);
  });
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
