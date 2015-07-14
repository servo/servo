document.write("<script>alert_assert('Pass 1 of 2');</script>");

var s = document.createElement('script');
s.innerText = "alert_assert('Pass 2 of 2');";
document.body.appendChild(s);
