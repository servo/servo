
var fs = require("fs")
,   pth = require("path")
,   id = process.argv[2]
;

if (!id) {
    console.log("Missing ID");
    process.exit(1);
}

console.log(JSON.parse(fs.readFileSync(pth.join(__dirname, "id2path.json"), "utf8"))[id]);
