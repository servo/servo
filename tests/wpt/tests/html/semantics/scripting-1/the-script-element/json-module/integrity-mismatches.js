import json from "./data.json" with { type: "json" };
window.mismatchesLog.push(`integrity-mismatches,json:${json.answer}`);
