function createDOMTree(node, siblings, depth) {
    if (!depth)
        return;
    for (var i=0; i<siblings; i++) {
        var div = document.createElement("div");
        node.appendChild(div);
        createDOMTree(div, siblings, depth-1);
    }
}

function createDeepDOMTree() {
    createDOMTree(document.body, 2, 10);
}

function createShallowDOMTree() {
    createDOMTree(document.body, 10, 2);
}

function createRegularDOMTree() {
    createDOMTree(document.body, 4, 4);
}

function forceStyleRecalc(node) {
    node.offsetTop; // forces style recalc
}

function applyCSSRule(rule) {
    var css = document.createElement("style");
    css.type = "text/css";
    css.innerHTML = rule;
    document.body.appendChild(css);
    return css;
}
