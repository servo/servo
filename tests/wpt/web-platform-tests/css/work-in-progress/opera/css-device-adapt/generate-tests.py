#!/usr/bin/env python

import argparse
import json
import os
import re

class TestGenerator:
    """Class used for generating an .html or .xht file for each test defined by
       entries in the json file."""

    def __init__(self, harness_url, vendor, template_file, output_dir, infile):
        self.harness_url = harness_url
        self.vendor = vendor
        self.output_dir = output_dir
        extMatch = re.search("\.([^\.]+)$", template_file.name)
        if extMatch == None:
            self.ext = "html"
        else:
            self.ext = extMatch.group(1)
        self.test_file_template = template_file.read()
        template_file.close()
        self.tests = json.load(infile)
        self.spec_url = self.tests["SPEC_URL"]

    def getTestFileName(self, file_prefix, idx):
        return os.path.join(self.output_dir, file_prefix + str(idx+1).rjust(3, '0') + "." + self.ext)

    def getSpecURL(self):
        return self.spec_url

    def getSpecSectionURL(self, spec_section):
        return self.getSpecURL() + spec_section

    def getVendorPrefix(self):
        if self.vendor is "":
            return ""
        return "-"+self.vendor+"-"

    def getHarnessURL(self):
        return self.harness_url

    def generateTest(self, file_prefix, spec_section, idx, test):
        file_name = self.getTestFileName(file_prefix, idx)
        print file_name
        test["SPEC_SECTION"] = self.getSpecSectionURL(spec_section)
        test["VENDOR_PREFIX"] = self.getVendorPrefix()
        test["HARNESS_URL"] = self.getHarnessURL()

        if "VIEWPORT_DESC" in test:
            test["TEST_CSS"] = "@{VENDOR_PREFIX}viewport {{ {VIEWPORT_DESC} }}".format(**test)
        elif "TEST_CSS" in test:
            test["TEST_CSS"] = re.sub("viewport", self.getVendorPrefix()+"viewport", test["TEST_CSS"])
        else:
            test["TEST_CSS"] = ""

        if "TEST_FUNCTION" in test and self.vendor:
            test["TEST_FUNCTION"] = re.sub("VIEWPORT_RULE", self.vendor.upper() + "_VIEWPORT_RULE", test["TEST_FUNCTION"])

        file = open(file_name, "w")
        file.write(self.test_file_template.format(**test))
        file.close()

    def generateGroup(self, group):
        file_prefix = group["FILE_PREFIX"]
        spec_section = group["SPEC_SECTION"]

        for i,test in enumerate(group["TESTS"]):
            self.generateTest(file_prefix, spec_section, i, test)

    def generateTests(self):
        for group in self.tests["GROUPS"]:
            self.generateGroup(group)

def main():
    parser = argparse.ArgumentParser(description="Generate the CSS Device Adaptation testsuite.", formatter_class=argparse.ArgumentDefaultsHelpFormatter)
    parser.add_argument("-u", "--harness-url", default="http://w3c-test.org/resources/", help="The URL of the directory where the testharness files are found.")
    parser.add_argument("-v", "--vendor", default="", help="The vendor string used for @viewport and corresponding CSSOM names. For instance 'o' for @-o-viewport. By default, no prefix will be added.")
    parser.add_argument("-t", "--template", type=file, default="template.html", help="Select template file to use (HTML5 or XHTML output).")
    parser.add_argument("-o", "--output-dir", default="", help="Choose the output directory of the generated test files.")
    parser.add_argument("infile", type=file, help="The input .json file which contains the tests.")

    args = parser.parse_args()

    generator = TestGenerator(args.harness_url, args.vendor, args.template, args.output_dir, args.infile)

    print "\nGenerating tests for " + generator.getSpecURL() + ":\n"

    generator.generateTests()

    print "\nDone.\n"

if __name__ == '__main__':
    main()
