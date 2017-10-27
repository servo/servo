﻿## Implement Mutation Testing on Servo Parallel Browsing Project


The motivation for mutation testing is to test the breadth coverage of tests for source code. Faults (or mutations) are automatically seeded into the code, then tests are run. If tests fail then the mutation is killed, if the tests pass then the mutation lived. The quality of tests can be gauged from the percentage of mutations killed.

For more info refer [Wiki page](https://en.wikipedia.org/wiki/Mutation_testing).

In this project, mutation testing is used to test the coverage of WPT for Servo's browsing engine.

### Mutation Strategy
This version of mutation testing consists of a Python script that finds random uses of && in Servo's code base and replaces them by ||. The expectation from the WPT tests is to catch this mutation and result in failures when executed on the corresponding code base.

### Test Run Strategy
The mutation test aims to run only tests which are concerned with the mutant. Therefore part of WPT test is related to the source code under mutation is invoked. For this it requires a test mapping in source folders.

#### test_mapping.json
The file test_mapping.json is used to map the source code to their corresponding WPT tests. The user must maintain a updated version of this file in the path where mutation testing needs to be performed. Additionally, the test_mapping.json will only consist of maps of source codes that are present in the current directory. Hence, each folder will have a unique test_mapping.json file. Any source code files that may be present in a path but are not mapped to a WPT in test_mapping.json will not be covered for mutation testing.

### Sample test_mapping.json format
A sample of test_mapping.json is as shown below:

```
{
  "xmlhttprequest.rs": [
    "XMLHttpRequest"
  ],  
  "range.rs": [
    "dom/ranges"
  ]
}
```

Please ensure that each folder that requires a mutant to be generated consists of test_mapping.json file so that the script can function as expected.

### Basic Execution Flow
The implementation of mutation testing is as follows:
1. The script is called from the command line, it searches through the path entered by user for test_mapping.json.
2. If found, it reads the json file and parses one componenent of source file to generate mutants. 
3. The corresponding WPT tests are run for each mutant and the test results are logged.
4. Once all WPT are run for the first source file, the mutation continues for other source files mentioned in the json file and runs their corresponding WPT tests.
5. Once it has completed executing mutation testing for the entered path, it repeats the above procedure for sub-paths present inside the entered path.


### Running Mutation test
The mutation tests can be run by running the below command from the servo directory on the command line interface:

`python python/servo/mutation/init.py <Mutation path>`

Eg. `python python/servo/mutation/init.py components/script/dom`

### Running Mutation Test from CI

The CI script for running mutation testing is present in /etc/ci folder. It can be called by executing the below command from the CLI:

`python /etc/ci/mutation_test.py`