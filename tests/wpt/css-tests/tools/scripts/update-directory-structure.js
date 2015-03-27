
// convert from old-style test structure to new style

var fs = require("fs")
,   pth = require("path")
,   jsdom = require("jsdom")
,   mkdirp = require("mkdirp").sync
,   testDir = pth.join(__dirname, "../..")
,   MAX_DEPTH = 3
,   id2path = {}
,   limitDepth = {
        parsing:    true
    }
;

var sections = {
    html:       "http://www.w3.org/html/wg/drafts/html/master/Overview.html"
,   canvas2d:   "http://www.w3.org/html/wg/drafts/2dcontext/html5_canvas/Overview.html"
,   microdata:  "http://www.w3.org/html/wg/drafts/microdata/master/Overview.html"
};

function walkTree ($, $el, list) {
    $el.find("> li").each(function () {
        var $li = $(this)
        ,   $a = $li.find("> a").first()
        ;
        // skip sections that don't have a number
        if (!/^\s*\d+/.test($a.text())) return;
        var href = $a.attr("href").replace(/^.*#/, "")
        ,   def = {
                id: href.toLowerCase()
                        .replace(/[^a-z0-9\-]/g, "-")
                        .replace(/\-{2,}/g, "-")
                        .replace(/(?:^\-|\-$)/g, "")
            ,   original_id: href
            }
        ,   $ol = $li.find("> ol").first()
        ;
        if ($ol.length) {
            def.children = [];
            walkTree($, $ol, def.children);
        }
        list.push(def);
    });
}

function extractSections (sec, secDir, spec, cb) {
    jsdom.env(
        spec
    ,   function (err, window) {
            if (err) return cb(err);
            jsdom.jQueryify(window, "https://ajax.googleapis.com/ajax/libs/jquery/1.8.3/jquery.min.js", function (window, $) {
                if (!$) return cb("$ was not defined");
                var $root = $("body > ol.toc").first()
                ,   tree = []
                ;
                walkTree($, $root, tree);
                cb(null, tree, sec, secDir);
            }
        );
    });
}

function makeID2Path (base, tree) {
    for (var i = 0, n = tree.length; i < n; i++) {
        var sec = tree[i];
        id2path[sec.original_id] = base;
        if (sec.children && sec.children.length) makeID2Path(base, sec.children);
    }
}

function makeDirs (base, tree, depth) {
    console.log("Making " + base + " at depth " + depth);
    for (var i = 0, n = tree.length; i < n; i++) {
        var sec = tree[i]
        ,   path = pth.join(base, sec.id)
        ;
        mkdirp(path);
        fs.writeFileSync(pth.join(path, ".gitkeep"), "", "utf8");
        id2path[sec.original_id] = path;
        if (sec.id !== sec.original_id) {
            fs.writeFileSync(pth.join(path, "original-id.json"), JSON.stringify({ original_id: sec.original_id}), "utf8");
        }
        if (sec.children && sec.children.length) {
            if (depth === MAX_DEPTH || limitDepth[sec.id]) {
                fs.writeFileSync(pth.join(path, "contains.json"), JSON.stringify(sec.children, null, 4), "utf8");
                makeID2Path(path, sec.children);
            }
            else {
                makeDirs(path, sec.children, depth + 1);
            }
        }
    }
}

for (var sec in sections) {
    var secDir = pth.join(testDir, sec);
    mkdirp(secDir);
    console.log("Launching extraction for " + sec);
    extractSections(sec, secDir, sections[sec], function (err, toc, sec, secDir) {
        if (err) return console.log("ERROR: " + err);
        makeDirs(secDir, toc, 1);
        for (var k in id2path) id2path[k] = id2path[k].replace(testDir + "/", "");
        fs.writeFileSync(pth.join(__dirname, "id2path.json"), JSON.stringify(id2path, null, 4), "utf8");
    });
}
