import styleSheet from "./basic.css" with { type: "css" };
window.matchesLog.push(`integrity-mismatches,css:${styleSheet.cssRules[0].cssText}`);
