export async function loadIframe(src = "/screen-orientation/resources/blank.html") {
  const iframe = document.createElement("iframe");
  iframe.src = src;
  document.body.appendChild(iframe);
  return new Promise(r => {
    if (iframe.contentDocument.readyState === "complete") {
      return r(iframe);
    }
    iframe.onload = () => r(iframe);
  });
}

export function getOppositeOrientation() {
  const { type: currentOrientation } = screen.orientation;
  const isPortrait = currentOrientation.includes("portrait");
  return isPortrait ? "landscape" : "portrait";
}
