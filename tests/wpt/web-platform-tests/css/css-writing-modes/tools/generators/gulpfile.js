'use strict';

var browserSync = null;
var gulp = require("gulp");
var ejs = require("gulp-ejs");
var rename = require("gulp-rename");
var minimist = require('minimist');
var argv = minimist(process.argv.slice(2));

gulp.task("default", [
  "orthogonal-parent-shrink-to-fit",
]);

gulp.task("test", ["browser-sync", "watch"]);

gulp.task("watch", function () {
  gulp.watch("orthogonal-parent-shrink-to-fit.ejs", ["orthogonal-parent-shrink-to-fit"]);
});

gulp.task("browser-sync", function () {
  if (!browserSync)
    browserSync = require("browser-sync");
  browserSync({
    server: {
      baseDir: "../../..",
      directory: true,
    },
    startPath: "css-writing-modes-3/",
  });
});

function reload() {
  if (browserSync)
    browserSync.reload();
}

gulp.task("server", function () {
  var connect = require("connect");
  var serveIndex = require("serve-index");
  var serveStatic = require("serve-static");
  var directory = "../../..";
  var port = 8000;
  connect()
    .use(serveIndex(directory))
    .use(serveStatic(directory))
    .listen(port);
  console.log("Listening on port " + port);
})

gulpTaskFromTemplateWithAffixes("orthogonal-parent-shrink-to-fit", "-001", -1, 24);

gulp.task("update", function () {
  const unicodeData = require('./unicode-data.js');
  unicodeData.copyToLocal();
});

function gulpTaskFromTemplateWithAffixes(name, suffix, min, lim) {
  if (argv.nocombo && min < 0)
    min = 0;
  if (argv.nochild && lim > 0)
    lim = 0;
  gulp.task(name, function () {
    for (var i = min; i < lim; ++i) {
      gulp.src(name + ".ejs")
        .pipe(ejs({ index: i }))
        .pipe(rename(name + suffix + affixFromIndex(i) + ".html"))
        .pipe(gulp.dest("../.."));
    }
    reload();
  });
}

function affixFromIndex(index) {
  if (index < 0)
    return "";
  if (index >= 26)
    throw new Error("Affix index too large (" + index + ")");
  return String.fromCharCode("a".charCodeAt(0) + index);
}
