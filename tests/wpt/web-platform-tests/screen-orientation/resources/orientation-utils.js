/**
 *
 * @param {object} options
 * @param {string} options.src - The iframe src
 * @param {Window} options.context - The browsing context in which the iframe will be created
 * @param {string} options.sandbox - The sandbox attribute for the iframe
 * @returns
 */
export async function attachIframe(options = {}) {
  const { src, context, sandbox, allowFullscreen } = {
    ...{
      src: "about:blank",
      context: self,
      allowFullscreen: true,
      sandbox: null,
    },
    ...options,
  };
  const iframe = context.document.createElement("iframe");
  if (sandbox !== null) iframe.sandbox = sandbox;
  iframe.allowFullscreen = allowFullscreen;
  await new Promise((resolve) => {
    iframe.onload = resolve;
    iframe.src = src;
    context.document.body.appendChild(iframe);
  });
  return iframe;
}

export function getOppositeOrientation() {
  return screen.orientation.type.startsWith("portrait")
    ? "landscape"
    : "portrait";
}

export function makeCleanup(
  initialOrientation = screen.orientation?.type.split(/-/)[0]
) {
  return async () => {
    if (initialOrientation) {
      try {
        await screen.orientation.lock(initialOrientation);
      } catch {}
    }
    screen.orientation.unlock();
    requestAnimationFrame(async () => {
      try {
        await document.exitFullscreen();
      } catch {}
    });
  };
}
