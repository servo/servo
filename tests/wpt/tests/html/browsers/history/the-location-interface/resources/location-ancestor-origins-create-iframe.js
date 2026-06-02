// The different actions that we send to a cross-origin document via postMessage (since they may be/are site isolated)
const Actions = {
  addFrame: "addFrame",
  changeReferrerPolicy: "changeReferrerPolicy",
  insertMetaElement: "insertMetaElement",
}

async function configAndNavigateIFrame(iframe, { src, name, referrerPolicy, options }) {
  iframe.name = name;
  iframe.referrerPolicy = referrerPolicy;
  if (options?.sandbox) {
    // postMessage needs to work
    iframe.setAttribute("sandbox", "allow-scripts");
  }
  document.body.appendChild(iframe);
  await new Promise((resolve) => {
    iframe.addEventListener("load", resolve, { once: true });
    iframe.src = src;
  });
}
