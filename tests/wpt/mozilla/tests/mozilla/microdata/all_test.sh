for testFile in `ls *.html`
do
  ../../../../../../mach run $testFile
done
