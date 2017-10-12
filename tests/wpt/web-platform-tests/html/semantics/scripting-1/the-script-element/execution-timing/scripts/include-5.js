log('include-5 before removing scripts');
var scripts=[].slice.call(document.getElementsByTagName('script'), 3);
for(var i = 0; i < scripts.length; i++) {
    var s = scripts[i];
    s.parentNode.removeChild(s);
}
log('include-5 after removing scripts');
