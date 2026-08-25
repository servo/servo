function generateParserDelay(seconds = 1) {
  seconds += (Math.random() / 10.0);
  document.write(`
    <script src="/loading/resources/dummy.js?pipe=trickle(d${seconds})"></script>
  `);
}
