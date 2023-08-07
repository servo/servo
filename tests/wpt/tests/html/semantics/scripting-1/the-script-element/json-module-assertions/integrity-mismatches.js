import json "./data.json" assert { type: "json" };
window.mismatchesLog.push(`integrity-mismatches,json:${json.answer}`);
