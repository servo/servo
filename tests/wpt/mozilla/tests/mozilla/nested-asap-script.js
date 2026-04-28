t.step(function () {
    var s = document.createElement("script");
    s.setAttribute("async", "");
    s.src = "data:text/javascript,t.done()";
    document.body.appendChild(s);
});
