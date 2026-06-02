document.getElementById("start-button").onclick = () => {
  document.getElementById("prep").style.display = "none";
  document.getElementById("pick-device").style.display = "block";
};
document.getElementById("prompt-button-prep").onclick = () => {
  v.remote
    .prompt()
    .then(() => {})
    .catch(() => {});
};
