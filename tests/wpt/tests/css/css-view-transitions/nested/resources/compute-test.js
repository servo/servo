failIfNot(document.startViewTransition, "Missing document.startViewTransition");

function add_rule() {
    const style = document.createElement("style");
    style.innerHTML = "@view-transition { navigation: auto }";
    document.head.append(style);
}

onload = async() => {
    document.documentElement.classList.add("vt-old");
    const transition = document.startViewTransition(() => {
        document.documentElement.classList.add("vt-new");
        document.documentElement.classList.remove("vt-old");
    });
    transition.finished.then(() => {
        document.documentElement.classList.remove("vt-new");
        document.documentElement.classList.remove("vt-old");
    });
    transition.ready.then(() => takeScreenshot());
}

