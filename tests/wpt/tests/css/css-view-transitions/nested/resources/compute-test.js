failIfNot(document.startViewTransition, "Missing document.startViewTransition");

function add_rule() {
    const style = document.createElement("style");
    style.innerHTML = "@view-transition { navigation: auto }";
    document.head.append(style);
}

const mode = new URLSearchParams(location.search).get("vtmode");
if (mode === "crossdoc") {
    onload = () => {
        const url = new URL(location.href);
        url.searchParams.set("vtmode", "crossdoc-newpage");
        location.replace(url.href);
    };
    add_rule();
} else if (mode === "crossdoc-newpage") {
    document.documentElement.classList.add("vt-new");
    add_rule();
    takeScreenshot();
} else {
    onload = async() => {
        const transition = document.startViewTransition(() => {
            document.documentElement.classList.add("vt-new");
        });
        transition.finished.then(() => {
            document.documentElement.classList.remove("vt-new");
        });
        transition.ready.then(() => takeScreenshot());
    }
}

