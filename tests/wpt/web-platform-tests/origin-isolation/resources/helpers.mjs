export function insertIframe(hostname, header) {
  const url = new URL("send-origin-isolation-header.py", import.meta.url);
  url.hostname = hostname;

  if (header !== undefined) {
    url.searchParams.set("header", header);
  }

  const iframe = document.createElement("iframe");
  iframe.src = url.href;

  return new Promise((resolve, reject) => {
    iframe.onload = () => resolve(iframe.contentWindow);
    iframe.onerror = () => reject(new Error(`Could not load ${iframe.src}`));
    document.body.append(iframe);
  });
}

// This function is coupled to ./send-origin-isolation-header.py, which ensures
// that sending such a message will result in a message back.
export async function sendWasmModule(frameWindow) {
  frameWindow.postMessage(await createWasmModule(), "*");
  return waitForMessage(frameWindow);
}

export async function sendWasmModuleBetween(frameWindow, indexIntoParentFrameOfDestination) {
  frameWindow.postMessage({ command: "send WASM module", indexIntoParentFrameOfDestination }, "*");
  return waitForMessage(frameWindow);
}

export async function accessDocumentBetween(frameWindow, indexIntoParentFrameOfDestination) {
  frameWindow.postMessage({ command: "access document", indexIntoParentFrameOfDestination }, "*");
  return waitForMessage(frameWindow);
}

// This function is coupled to ./send-origin-isolation-header.py, which ensures
// that sending such a message will result in a message back.
export async function setBothDocumentDomains(frameWindow) {
  // By setting both this page's document.domain and the iframe's document.domain to the same
  // value, we ensure that they can synchronously access each other, unless they are
  // origin-isolated.
  // NOTE: document.domain being unset is different than it being set to its current value.
  // It is a terrible API.
  document.domain = document.domain;

  frameWindow.postMessage({ command: "set document.domain", newDocumentDomain: document.domain }, "*");
  const whatHappened = await waitForMessage(frameWindow);
  assert_equals(whatHappened, "document.domain is set");
}

function waitForMessage(expectedSource) {
  return new Promise(resolve => {
    const handler = e => {
      if (e.source === expectedSource) {
        resolve(e.data);
        window.removeEventListener("message", handler);
      }
    };
    window.addEventListener("message", handler);
  });
}

// Any WebAssembly.Module will work fine for our tests; we just want to find out if it gives
// message or messageerror. So, we reuse one from the /wasm/ tests.
async function createWasmModule() {
  const response = await fetch("/wasm/serialization/module/resources/incrementer.wasm");
  const ab = await response.arrayBuffer();
  return WebAssembly.compile(ab);
}
