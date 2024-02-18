function generateParserDelay(seconds = 1) {
  document.write(`
    <script src="/loading/resources/dummy.js?pipe=trickle(d${seconds})"></script>
  `);
}
