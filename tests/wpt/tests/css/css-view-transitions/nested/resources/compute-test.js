failIfNot(document.startViewTransition, "Missing document.startViewTransition");

function runTest() {
    document.startViewTransition().ready.then(() => takeScreenshot());
}

onload = () => requestAnimationFrame(() => requestAnimationFrame(runTest));
