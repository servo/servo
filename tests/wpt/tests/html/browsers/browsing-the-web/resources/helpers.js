// This file contains general helpers for navigation/history tests. The goal is
// to make tests more imperative and ordered, instead of requiring lots of
// nested callbacks and jumping back and forth. However,
// html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
// might be even better at that, so prefer that when you can.
//
// TODO(domenic): consider unifying with
// overlapping-navigations-and-traversals/resources/helpers.mjs.

window.openWindow = (url, t) => {
  const w = window.open(url);
  t?.add_cleanup(() => w.close());

  return new Promise(resolve => {
    w.addEventListener("load", () => resolve(w), { once: true });
  });
};

window.addIframe = (url = "/common/blank.html", doc = document) => {
  const iframe = doc.createElement("iframe");
  iframe.src = url;
  doc.body.append(iframe);

  return new Promise(resolve => {
    iframe.addEventListener("load", () => resolve(iframe), { once: true });
  });
};

window.addSrcdocIframe = async () => {
  const iframe = document.createElement("iframe");
  iframe.srcdoc = `<script>window.parent.postMessage("srcdoc ready", "*")</scr` + `ipt>`;
  document.body.append(iframe);

  assert_equals(await waitForMessage(iframe.contentWindow), "srcdoc ready");

  return iframe;
};

window.waitToAvoidReplace = t => {
  return new Promise(resolve => t.step_timeout(resolve, 0));
};

window.waitForIframeLoad = iframe => {
  return new Promise(resolve => {
    iframe.addEventListener("load", () => resolve(), { once: true });
  });
};

window.waitForMessage = expectedSource => {
  return new Promise(resolve => {
    window.addEventListener("message", ({ source, data }) => {
      if (source === expectedSource) {
        resolve(data);
      }
    });
  });
};

window.waitForHashchange = w => {
  return new Promise(resolve => {
    w.addEventListener("hashchange", () => resolve(), { once: true });
  });
};

window.srcdocThatPostsParentOpener = text => {
  return `
    <p>${text}</p>
    <script>
      window.onload = () => {
        window.top.opener.postMessage('ready', '*');
      };
    <\/script>
  `;
};

window.failOnMessage = expectedSource => {
  return new Promise((_, reject) => {
    window.addEventListener("message", ({ source, data }) => {
      if (source === expectedSource) {
        reject(new Error(`Received message "${data}" but expected to receive no message`));
      }
    });
  });
};
