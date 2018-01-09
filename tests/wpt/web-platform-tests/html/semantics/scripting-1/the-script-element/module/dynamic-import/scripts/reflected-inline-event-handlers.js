window.dummyDiv.setAttribute("onclick", `import('../../imports-a.js?label=' + window.label).then(window.continueTest, window.errorTest)`);
window.dummyDiv.onclick();
