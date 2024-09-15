failIfNot(document.startViewTransition, "Missing document.startViewTransition");

function add_rule() {
    const style = document.createElement("style");
    style.innerHTML = "@view-transition { navigation: auto }";
    document.head.append(style);
}

onload = async() => {
    const transition = document.startViewTransition(() => {
        document.documentElement.classList.add("vt-new");
    });
    transition.finished.then(() => {
        document.documentElement.classList.remove("vt-new");
    });
    transition.ready.then(() => takeScreenshot());
}

