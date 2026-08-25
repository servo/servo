import os
import re

# Generate an HTML file for each .test file in the current directory
#

TEST_LIST_FILE = '00_test_list.txt';
TEMPLATE = 'template.html';

def genHTML(template, test):
	contents = re.sub('___TEST_NAME___', "'" + test + "'", template);
	filename = test + '.html';
	print "Generating " + filename;
	with open(test + '.html', 'w') as f:
		f.write(contents);
	return filename;


def process_test_files(template):
	generated = [];
	files = os.listdir(os.getcwd());
	for file in files:
		found = re.search('(^[^.].*)\.test$', file);
		if found:
			generated.append(genHTML(template,found.group(1)));
	return generated;

def readTemplate():
	contents = None;
	with open(TEMPLATE, 'r') as f:
		contents = f.read();
	return contents;


template = readTemplate();
if (template):
	test_list = process_test_files(template);
	print "Generating " + TEST_LIST_FILE;
	with open(TEST_LIST_FILE, 'w') as f:
		for item in test_list:
			f.write(item + '\n');
else:
	print "Couldn't find template file: " + TEMPLATE;
