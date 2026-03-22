const fs = require("fs");
const fse = require('fs-extra');
const path = require("path");


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


const addHarnessToTestsHeader = async(testsPath,testsListPath) =>{
  var files = fs.readFileSync(testsListPath).toString().split("\n");
  var numberOfTestFiles = 0;
  for(var i=0; i<files.length ; i++){
    var fileExtension = files[i].split('.').pop();
    filename = path.join(testsPath,files[i]);
    if(fs.existsSync(filename)){
      if(fileExtension == "html" || fileExtension == "htm"){
         var content = fs.readFileSync(filename, 'utf8');
         content = content.replace("<head>", '<head> \n<script src="/resources/testharness.js"></script> \n<script src="/resources/testharnessreport.js"></script> \n');
         var file = fs.openSync(filename,'r+');
         fs.writeSync(file, content);
         numberOfTestFiles += 1;
      }
    }
  }
  return numberOfTestFiles;
}


(async () => {

  const testDir = process.argv[2] || DEFAULT_TEST_DIR;

  // Files that will be overwritten in the original webgl test suite 
  const PRE_TEST_NAME = "js-test-pre.js";
  const UNIT_TEST_NAME = "unit.js";

  const RESOURCES = path.join( __dirname ,"resources");
  const DEFAULT_TEST_DIR = "/webgl/";
  const DEFAULT_OUTPUT_DIR = ".";
  const SUB_DIR_NAME = "webgl";

  const testsPath = path.join(testDir, "conformance-suites");
  const v1_0_3_harnessDir = path.join(testsPath, "1.0.3");
  const preTestsPath = path.join(RESOURCES, PRE_TEST_NAME);
  const unitTestPath = path.join(RESOURCES, UNIT_TEST_NAME);
  let outputPath = process.argv[3] || DEFAULT_OUTPUT_DIR;
  outputPath = path.join(outputPath, SUB_DIR_NAME);
  const testsOutputPath = path.join(outputPath, "conformance-suite");
  const resourcesPath = path.join(testsOutputPath, "resources");
  const presTestDestinationPath = path.join(resourcesPath, "js-test-pre.js");
  const unitTestDestinationputPath = path.join(testsOutputPath, "conformance", "more", "unit.js");

  const testsListPath = path.join(RESOURCES, "list_all_tests")
  
  if (!(await readStats(SUB_DIR_NAME))) await makeDirectory(SUB_DIR_NAME);

  await fse.copy(v1_0_3_harnessDir, testsOutputPath);
  await fse.copy(preTestsPath, presTestDestinationPath);
  await fse.copy(unitTestPath, unitTestDestinationputPath);
  const numberOfTestFiles = await addHarnessToTestsHeader(testsOutputPath,testsListPath);
  console.log(`Total of ${numberOfTestFiles} webGl tests integrated`, testsListPath);
})();
