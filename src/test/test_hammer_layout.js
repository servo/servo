var divs = document.getElementsByTagName("div");
var div = divs[0];

var count = 1000000;
var start = new Date();
for (var i = 0; i < count; i++) {
  div.setAttribute('id', 'styled');
  div.getBoundingClientRect();
}
var stop = new Date();
window.alert((stop - start) / count * 1e6);
