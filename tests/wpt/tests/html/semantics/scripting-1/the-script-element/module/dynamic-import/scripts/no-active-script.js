"use strict";

document.querySelector("#button1").setAttribute("onclick", "import('../imports-a.js?label=button1').then(window.continueTest1, window.errorTest1)");

document.querySelector("#button2").setAttribute("onclick", "Promise.resolve(`import('../imports-a.js?label=button2')`).then(eval).then(window.continueTest2, window.errorTest2);");
