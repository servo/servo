## Implement Mutation Testing on Servo Parallel Browsing Project


The motivation for mutation testing is to test the breadth coverage of tests for source code. Faults (or mutations) are automatically seeded into the code, then tests are run. If tests fail then the mutation is killed, if the tests pass then the mutation lived. The quality of tests can be gauged from the percentage of mutations killed.

For more info refer [Wiki page](https://en.wikipedia.org/wiki/Mutation_testing).

Here Mutation testing is used to test the coverage of WPT for Servo's browser engine.

### Mutation Strategies
The mutation test consists of a Python script that mutates random lines in Servo's code base. The expectation from the WPT tests is to catch this bug caused by mutation and result in test failures.

There are few strategies to mutate source code in order to create bugs. The strategies are randomly picked for each file. Some of the strategies are:

* Change Conditional flow
* Delete if block
* Change Arithmetic operations

#### How To Add a New Strategy?
Write new class inheriting the Strategy class in mutator.py and include it in get_strategies method. Override mutate method or provide replace strategy regex if it works with mutate method of Strategy class.

### Test Run Strategy
The mutation test aims to run only tests which are concerned with the mutant. Therefore part of WPT test is related to the source code under mutation is invoked. For this it requires a test mapping in source folders.

#### test_mapping.json
The file test_mapping.json is used to map the source code to their corresponding WPT tests. The user must maintain a updated version of this file in the path where mutation testing needs to be performed. Additionally, the test_mapping.json will only consist of maps of source codes that are present in the current directory. Hence, each folder will have a unique test_mapping.json file. Any source code files that may be present in a path but are not mapped to a WPT in test_mapping.json will not be covered for mutation testing.

#### Sample test_mapping.json format
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

Please ensure that each folder that requires a mutant to be generated consists of test_mapping.json file so that the script can function as expected. Wildcards are not allowed in test_mapping.json.

If we want to run mutation test for a source path then there should be test_mapping.json in that path and all the subdirectories which has source files.

Eg: There should be test mapping in following folders if we run mutation test on 'components/script' path.
* components/script/test_mapping.json
* components/script/dom/test_mapping.json
* components/script/task_source/test_mapping.json
* components/script/dom/bindings/test_mapping.json
* ...

### Running Mutation test
The mutation tests can be run by running the below command from the servo directory on the command line interface:

`python python/servo/mutation/init.py <Mutation path>`

Eg. `python python/servo/mutation/init.py components/script/dom`

### Running Mutation Test from CI
The CI script for running mutation testing is present in /etc/ci folder. It can be called by executing the below command from the CLI:

`./etc/ci/mutation_test.sh`

### Execution Flow
1. The script is called from the command line, it searches for test_mapping.json in the path entered by user.
2. If found, it reads the json file and parses it, gets source file to tests mapping. For all source files in the mapping file, it does the following.
3. If the source file does not have any local changes then it mutates at a random line using a random strategy. It retries with other strategies if that strategy could not produce any mutation.
4. The code is built and the corresponding WPT tests are run for this mutant and the test results are logged.
5. Once it has completed executing mutation testing for the entered path, it repeats the above procedure for sub-paths present inside the entered path.

### Test Summary
At the end of the test run the test summary displayed which looks like this:
```
Test Summary:
Mutant Killed (Success)         25
Mutant Survived (Failure)       10
Mutation Skipped                1
Unexpected error in mutation    0
```

* Mutant Killed (Success): The mutant was successfully killed by WPT test suite.
* Mutant Survived (Failure): The mutation has survived the WPT Test Suite, tests in WPT could not catch this mutation.
* Mutation Skipped: Files is skipped for mutation test due to the local changes in that file.
* Unexpected error in mutation: Mutation test could not run due to unexpected failures. (example: if no && preset in the file to replace)
