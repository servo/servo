import os
import sys
import json
import fnmatch

TEST_DIR = "/webvtt/"
CATEGORIES_FILE = "../categories.json"

class Test:
    def __init__(self, file, name, status, message):
        self.file = file
        self.name = name
        self.status = status
        self.message = message
        self.passed = status == 'PASS'
        self.categories = []

    @classmethod
    def from_json(cls, json):
        file = json["test"]
        if not file.startswith(TEST_DIR):
            return []
        file = file[len(TEST_DIR):]

        status = json["status"]
        message = json["message"]

        tests = []

        for test in json["subtests"]:
            name = test["name"]
            if status == 'OK':
                test_status = test["status"]
                test_message = test["message"]
            else:
                test_status, test_message = status, message

            tests.append(Test(file, name, test_status, test_message))

        return tests

class Category:
    def __init__(self, names):
        self.names = set(names)
        self.tests = {}

    @classmethod
    def from_json(cls, json):
        return Category(json)

    def add_test(self, name, test):
        self.tests[test] = name

    def __contains__(self, name):
        return name in self.names

def parse_results(file):
    data = json.load(file)

    results = data["results"]
    tests = []
    for result in results:
        tests += Test.from_json(result)

    return tests

def parse_categories(file, tests, categories = None, categories_map = None):
    data = json.load(file)
    basepath = os.path.dirname(file.name)

    categories = categories or []

    if categories_map:
        categories_map = dict(categories_map)
    else:
        categories_map = {}

    if ":categories" in data:
        for cat_data in data[":categories"]:
            category = Category.from_json(cat_data)

            categories.append(category)
            for name in category.names:
                categories_map[name] = category

    for pattern, category_name in data.items():
        if pattern.startswith(":"):
            continue
        category = categories_map[category_name]

        file_pattern = os.path.normpath(os.path.join(basepath, pattern))
        for test in tests:
            if fnmatch.fnmatch(test.name, file_pattern) or fnmatch.fnmatch(test.file, file_pattern):
                category.add_test(category_name, test)
                test.categories.append(category)

    if ":subcategories" in data:
        for subcat_name in data[":subcategories"]:
            path = os.path.join(basepath, subcat_name)
            file = open(path, "r")
            parse_categories(file, tests, categories, categories_map)

    return categories

def main(argv):
    if len(argv) == 1:
        if argv[0] == '-':
            results_file = sys.stdin
        else:
            results_file = open(argv[0], "r")
    else:
        print("USAGE: python3 categorize_results.py <file>")
        print("<file>\tA file containing wpt results. Or `-` for reading results from stdin.")
        return

    filepath = os.path.dirname(__file__)
    categories_path = os.path.join(filepath, CATEGORIES_FILE)
    categories_file = open(categories_path, "r")

    tests = parse_results(results_file)
    categories = parse_categories(categories_file, tests)

    for category in categories:
        tests_by_name = { name: [] for name in category.names }
        for test, name in category.tests.items():
            tests_by_name[name].append(test)

        for name in category.names:
            test_group = tests_by_name[name]
            amount = len(test_group)
            if amount == 0:
                continue
            passed = sum(1 for test in test_group if test.passed)
            print("{}:\t{}/{} - {}%".format(name, passed, amount, round(passed / amount * 100, 2)))

if __name__ == "__main__":
    main(sys.argv[1:])
