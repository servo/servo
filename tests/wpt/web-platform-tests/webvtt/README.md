# WebVTT Tests

## Categorization

Dependencies:
* Python 3

Once you have run the tests with `/tools/runner/index.html`, press the "Download
JSON results" button to download a file containing the results. You can then run
the categorization tool to get a better overview of the test results:

```bash
$ python3 tools/categorize_results.py runner-results.json
```
