// import()s in an event handler are resolved relative to the document base.
window.dummyDiv.setAttribute("onclick", `import('./imports-a.js?label=' + window.label).then(window.continueTest, window.errorTest)`);
window.dummyDiv.click(); // different from **on**click()
