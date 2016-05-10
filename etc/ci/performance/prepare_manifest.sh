# TP5 manifest uses `localhost`, but our local server probably don't use port 80
sed 's/localhost\/page_load_test\/tp5n/localhost:8000\/page_load_test/g' ./page_load_test/tp5o.manifest > ./page_load_test/tp5o_8000.manifest

