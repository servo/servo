// This script is executed after |scr1| and |scr2| are inserted into DOM
// before their execution (if not blocked by CSP).
if (document.getElementById("scr1")) {
  document.getElementById("scr1").innerText =
    "log1 += 'scr1 at #execute-the-script-block';";
}
if (document.getElementById("scr2")) {
  document.getElementById("scr2").innerText =
    "log2 += 'scr2 at #execute-the-script-block';";
}
