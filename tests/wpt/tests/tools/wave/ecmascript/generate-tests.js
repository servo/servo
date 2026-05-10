const fs = require("fs-extra");
const path = require("path");

const readDirectory = async directoryPath => {
  return new Promise((resolve, reject) => {
    fs.readdir(directoryPath, (error, files) => {
      if (error) {
        reject(error);
      }
      resolve(files);
    });
  });
};

const makeDirectory = async directoryPath => {
  return new Promise((resolve, reject) => {
    fs.mkdir(directoryPath, error => {
      if (error) {
        reject(error);
      }
      resolve();
    });
  });
};

const readStats = async path => {
  return new Promise((resolve, reject) => {
    fs.stat(path, (error, stats) => {
      if (error) {
        resolve(null);
      }
      resolve(stats);
    });
  });
};

const readFile = async path => {
  return new Promise((resolve, reject) => {
    fs.readFile(
      path,
      {
        encoding: "UTF-8"
      },
      (error, data) => {
        if (error) {
          reject(error);
        }
        resolve(data);
      }
    );
  });
};

const writeFile = async (path, data) => {
  return new Promise((resolve, reject) => {
    fs.writeFile(path, data, error => {
      if (error) {
        reject(error);
      }
      resolve();
    });
  });
};

const parseFrontmatter = src => {
  var start = src.indexOf("/*---");
  var end = src.indexOf("---*/");
  if (start === -1 || end === -1) return null;

  var match,
    includes = [],
    flags = {},
    negative = null;
  var frontmatter = src.substring(start + 5, end);

  match = frontmatter.match(/(?:^|\n)\s*includes:\s*\[([^\]]*)\]/);
  if (match) {
    includes = match[1].split(",").map(function f(s) {
      return s.replace(/^\s+|\s+$/g, "");
    });
  } else {
    match = frontmatter.match(/(?:^|\n)\s*includes:\s*\n(\s+-.*\n)/);
    if (match) {
      includes = match[1].split(",").map(function f(s) {
        return s.replace(/^[\s\-]+|\s+$/g, "");
      });
    }
  }

  match = frontmatter.match(/(?:^|\n)\s*flags:\s*\[([^\]]*)\]/);
  if (match) {
    match[1]
      .split(",")
      .map(function f(s) {
        return s.replace(/^\s+|\s+$/g, "");
      })
      .forEach(function(flag) {
        switch (flag) {
          case "onlyStrict":
            if (flags.strict) {
              console.error("flag conflict", src);
            }
            flags.strict = "always";
            break;
          case "noStrict":
            if (flags.strict) {
              console.error("flag conflict");
            }
            flags.strict = "never";
            break;
          case "module":
            flags.module = true;
            break;
          case "raw":
            flags.raw = true;
            break;
          case "async":
            flags.async = true;
            break;
          case "generated":
          case "non-deterministic":
          case "CanBlockIsTrue":
          case "CanBlockIsFalse":
            break;
          default:
            console.error("unrecocognized flag: " + flag, frontmatter);
            break;
        }
      });
  }

  match = frontmatter.match(/(?:^|\n)\s*negative:/);
  if (match) {
    var phase, type;
    frontmatter
      .substr(match.index + 9)
      .split("\n")
      .forEach(function(line) {
        var match = line.match(/\s+phase:\s*(\S+)/);
        if (match) {
          phase = match[1];
        }
        match = line.match(/\s+type:\s*(\S+)/);
        if (match) {
          type = match[1];
        }
      });
    if (!phase || !type) return null;
    negative = {
      phase: phase,
      type: type
    };
  }

  return {
    includes: includes,
    flags: flags,
    negative: negative,
    isDynamic: /dynamic-import/.test(frontmatter)
  }; // lol, do better
};

const getOutputPath = ({ testsPath, currentPath, outputPath }) => {
  return path.join(outputPath, path.relative(testsPath, currentPath));
};

// Tests that will freeze the runner
// ch15/15.4/15.4.4/15.4.4.15/15.4.4.15-3-14.js
// ch15/15.4/15.4.4/15.4.4.18/15.4.4.18-3-14.js
// ch15/15.4/15.4.4/15.4.4.20/15.4.4.20-3-14.js
// ch15/15.4/15.4.4/15.4.4.21/15.4.4.21-3-14.js
// ch15/15.4/15.4.4/15.4.4.22/15.4.4.22-3-14.js
const excludedTests = [
  /15\.4\.4\.15-3-14\.js/,
  /15\.4\.4\.18-3-14\.js/,
  /15\.4\.4\.20-3-14\.js/,
  /15\.4\.4\.21-3-14\.js/,
  /15\.4\.4\.22-3-14\.js/
];

let testCount = 0;

const generateTest = async ({
  testsPath,
  outputPath,
  currentPath,
  templateContent,
  iframeTemplateContent
}) => {
  if (!currentPath) currentPath = testsPath;
  let stats = await readStats(currentPath);
  if (stats.isDirectory()) {
    const outputDir = getOutputPath({
      testsPath,
      outputPath,
      currentPath
    });
    if (!(await readStats(outputDir))) await makeDirectory(outputDir);
    let files = await readDirectory(currentPath);
    for (let file of files) {
      await generateTest({
        currentPath: path.join(currentPath, file),
        outputPath,
        testsPath,
        templateContent,
        iframeTemplateContent
      });
    }
  } else {
    if (
      currentPath.indexOf(".js") === -1 ||
      excludedTests.some(regex => regex.test(currentPath))
    ) {
      return;
    }

    const jsRelativePath = path.relative(testsPath, currentPath);
    const jsOutputPath = path.join(outputPath, jsRelativePath);
    const htmlOutputPath = jsOutputPath.replace(".js", ".html");
    const iframeHtmlOutputPath = jsOutputPath.replace(".js", ".iframe.html");
    const jsSrc = await readFile(currentPath);
    const meta = parseFrontmatter(jsSrc);
    const includes = (meta && meta.includes) || [];
    const testContent = replacePlaceholders(templateContent, {
      jsRelativePath,
      includes,
      iframeTestPath: `/${iframeHtmlOutputPath}`
    });

    const iframeTestContent = replacePlaceholders(iframeTemplateContent, {
      jsRelativePath,
      includes,
      iframeTestPath: `/${iframeHtmlOutputPath}`
    });

    await writeFile(htmlOutputPath, testContent);
    await writeFile(iframeHtmlOutputPath, iframeTestContent);
    await fs.copy(currentPath, jsOutputPath);
    testCount++;
  }
};

function replacePlaceholders(
  content,
  { jsRelativePath, includes, iframeTestPath }
) {
  content = content.replace(
    "{{ TEST_URL }}",
    "/ecmascript/tests/" + jsRelativePath
  );
  content = content.replace("{{ IFRAME_TEST_URL }}", iframeTestPath);
  content = content.replace(
    "{{ TEST_TITLE }}",
    jsRelativePath.split("/").pop()
  );
  content = content.replace(
    "{{ INCLUDES }}",
    includes
      .map(function(src) {
        return "<script src='/ecmascript/harness/" + src + "'></script>";
      })
      .join("\n")
  );
  return content;
}

(async () => {
  const ADAPTER_SCRIPT_NAME = "webplatform-adapter.js";
  const HTML_TEMPLATE_NAME = path.join(__dirname, "test-template.html");
  const IFRAME_HTML_TEMPLATE_NAME = path.join(
    __dirname,
    "test-template.iframe.html"
  );
  const DEFAULT_TEST_DIR = "./test262";
  const DEFAULT_OUTPUT_DIR = ".";
  const SUB_DIR_NAME = "ecmascript";

  const testDir = process.argv[2] || DEFAULT_TEST_DIR;
  const testsPath = path.join(testDir, "test");
  const harnessDir = path.join(testDir, "harness");
  let outputPath = process.argv[3] || DEFAULT_OUTPUT_DIR;
  outputPath = path.join(outputPath, SUB_DIR_NAME);
  const testsOutputPath = path.join(outputPath, "tests");
  const harnessOutputDir = path.join(outputPath, "harness");
  const adapterSourcePath = path.join(__dirname, ADAPTER_SCRIPT_NAME);
  const adapterDestinationPath = path.join(outputPath, ADAPTER_SCRIPT_NAME);

  if (!(await readStats(outputPath))) await makeDirectory(outputPath);

  console.log("Reading test templates ...");
  const templateContent = await readFile(HTML_TEMPLATE_NAME);
  const iframeTemplateContent = await readFile(IFRAME_HTML_TEMPLATE_NAME);
  console.log("Generating tests ...");
  await generateTest({
    testsPath,
    outputPath: testsOutputPath,
    templateContent,
    iframeTemplateContent
  });
  await fs.copy(adapterSourcePath, adapterDestinationPath);
  await fs.copy(harnessDir, harnessOutputDir);
  console.log(
    `Generated ${testCount} tests in directory ${outputPath} (${path.resolve(
      outputPath
    )})`
  );
})();
