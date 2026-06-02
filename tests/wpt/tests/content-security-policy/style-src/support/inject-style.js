document.write("<style>#test1 { display: none; }</style>");

var s = document.createElement('style');
s.textContent = "#test2 { display: none; }";
document.body.appendChild(s);
