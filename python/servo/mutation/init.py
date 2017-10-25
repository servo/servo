from os import listdir
from os.path import isfile, isdir, join
import json
import sys
import test

def get_folders_list(path):
    folder_list = []
    for filename in listdir(path):
    	if (isdir(join(path, filename))):
			folder_name = join(path,filename)
			folder_list.append(folder_name)
    return(folder_list)

def mutation_test_for(mutation_path):
    test_mapping_file = join(mutation_path, 'Test_mapping.json')
    if(isfile(test_mapping_file)):
        json_data = open(test_mapping_file).read()
        test_mapping = json.loads(json_data)

        for src_file in test_mapping.keys():
            test.mutation_test(join(mutation_path,src_file.encode('utf-8')), test_mapping[src_file])

        for folder in get_folders_list(mutation_path):
            mutation_test_for(folder)
    else:
		print ("This folder %s has no test mapping file." %(mutation_path))

mutation_test_for(sys.argv[1])
