// import()s in a timeout handler are resolved relative to the script.
setTimeout(`import('../../imports-a.js?label=' + window.label).then(window.continueTest, window.errorTest)`, 0);
