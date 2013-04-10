var divs = document.getElementsByTagName("div");
var div = divs[0];
for (var i = 0; i < 1000000; i++) {
  div.setAttribute('id', 'styled');
}

