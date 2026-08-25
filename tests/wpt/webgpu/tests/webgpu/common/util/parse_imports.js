/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/ /**
 * Parses all the paths of the typescript `import` statements from content
 * @param path the current path of the file
 * @param content the file content
 * @returns the list of import paths
 */export function parseImports(path, content) {const out = [];
  const importRE = /^import\s[^'"]*(['"])([./\w]*)(\1);/gm;
  let importMatch;
  while (importMatch = importRE.exec(content)) {
    const importPath = importMatch[2].replace(`'`, '').replace(`"`, '');
    out.push(joinPath(path, importPath));
  }
  return out;
}

function joinPath(a, b) {
  const aParts = a.split('/');
  const bParts = b.split('/');
  aParts.pop(); // remove file
  let bStart = 0;
  while (aParts.length > 0) {
    switch (bParts[bStart]) {
      case '.':
        bStart++;
        continue;
      case '..':
        aParts.pop();
        bStart++;
        continue;
    }
    break;
  }
  return [...aParts, ...bParts.slice(bStart)].join('/');
}