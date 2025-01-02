document.write("<script>log('Pass 1 of 2');</script>");

var s = document.createElement('script');
s.textContent = "log('Pass 2 of 2');";
document.body.appendChild(s);
