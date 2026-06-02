window.waitForLoad = (t, iframe, urlRelativeToThisDocument) => {
  return new Promise(resolve => {
    iframe.addEventListener("load", t.step_func(() => {
      assert_equals(iframe.contentWindow.location.href, (new URL(urlRelativeToThisDocument, location.href)).href);

      // Wait a bit longer to ensure all history stuff has settled, e.g. the document is "completely loaded"
      // (which happens from a queued task).
      setTimeout(resolve, 0);
    }), { once: true });
  });
};

window.waitForLoadAllowingIntermediateLoads = (t, iframe, urlRelativeToThisDocument) => {
  return new Promise(resolve => {
    const handler = t.step_func(() => {
      if (iframe.contentWindow.location.href === (new URL(urlRelativeToThisDocument, location.href)).href) {
        // Wait a bit longer to ensure all history stuff has settled, e.g. the document is "completely loaded"
        // (which happens from a queued task).
        setTimeout(resolve, 0);
        iframe.removeEventListener("load", handler);
      }
    });

    iframe.addEventListener("load", handler);
  });
};

window.waitForMessage = () => {
  return new Promise(resolve => {
    window.addEventListener("message", e => {
      resolve(e.data);
    }, { once: true });
  });
};

window.setupSentinelIframe = async (t) => {
  // If this iframe gets navigated by history.back(), then the iframe under test did not, so we did a replace.
  const sentinelIframe = document.createElement("iframe");
  sentinelIframe.src = "/common/blank.html?sentinelstart";
  document.body.append(sentinelIframe);
  t.add_cleanup(() => sentinelIframe.remove());

  await waitForLoad(t, sentinelIframe, "/common/blank.html?sentinelstart");

  sentinelIframe.src = "/common/blank.html?sentinelend";
  await waitForLoad(t, sentinelIframe, "/common/blank.html?sentinelend");

  return sentinelIframe;
};

window.checkSentinelIframe = async (t, sentinelIframe) => {
  // Go back. Since iframe should have done a replace, this should move sentinelIframe back, not iframe.
  history.back();
  await waitForLoad(t, sentinelIframe, "/common/blank.html?sentinelstart");
};

window.insertIframe = (t, url, name) => {
  const iframe = document.createElement("iframe");
  iframe.src = url;

  // In at least Chromium, window name targeting for form submission doesn't work if the name is set
  // after the iframe is inserted into the DOM. So we can't just have callers do this themselves.
  if (name) {
    iframe.name = name;
  }

  document.body.append(iframe);

  // Intentionally not including the following:
  //  t.add_cleanup(() => iframe.remove());
  // Doing so breaks some of the testdriver.js tests with "cannot find window" errors.
  return iframe;
};

// TODO(domenic): clean up other tests in the parent directory to use this.
window.absoluteURL = relativeURL => {
  return (new URL(relativeURL, location.href)).href;
};

// TODO(domenic): clean up other tests in the parent directory to use this.
window.codeInjectorURL = code => {
  return absoluteURL("resources/code-injector.html?pipe=sub(none)&code=" + encodeURIComponent(code));
};

window.changeURLHost = (url, newHost) => {
  const urlObj = new URL(url);
  urlObj.host = newHost;
  return urlObj.href;
};
