document.write("<style>#test1 { display: none; }</style>");

var s = document.createElement('style');
s.innerText = "#test2 { display: none; }";
document.body.appendChild(s);
