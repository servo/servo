// Append scripts that functions in this file depend on using a script element.
// Waiting for the load event ensures this is evaluated before code depending on it is executed.
const script = document.createElement('script');
script.src = "/common/rendering-utils.js";
const waitForScript = new Promise((resolve) => {
  script.addEventListener("load", resolve);
});
document.head.appendChild(script);

async function delayScreenshot() {
    await waitForScript;

    let frames = 4;
    for (let i = 0; i < frames; i++)
      await waitForAtLeastOneFrame();
    takeScreenshot();
}
