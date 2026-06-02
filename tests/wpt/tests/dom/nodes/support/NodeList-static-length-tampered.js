"use strict";

function makeStaticNodeList(length) {
    const fooRoot = document.createElement("div");

    for (var i = 0; i < length; i++) {
        const el = document.createElement("span");
        el.className = "foo";
        fooRoot.append(el);
    }

    document.body.append(fooRoot);
    return fooRoot.querySelectorAll(".foo");
}

const indexOfNodeList = new Function("nodeList", `
    const __cacheBust = ${Math.random()};

    const el = nodeList[50];

    let index = -1;

    for (var i = 0; i < 1e5 / 2; i++) {
        for (var j = 0; j < nodeList.length; j++) {
            if (nodeList[j] === el) {
                index = j;
                break;
            }
        }
    }

    return index;
`);

const arrayIndexOfNodeList = new Function("nodeList", `
    const __cacheBust = ${Math.random()};

    const el = nodeList[50];
    const {indexOf} = Array.prototype;

    for (var i = 0; i < 1e5; i++) {
        var index = indexOf.call(nodeList, el);
    }

    return index;
`);
