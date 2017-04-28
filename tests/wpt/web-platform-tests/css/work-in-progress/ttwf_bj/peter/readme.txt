Test the Web Forward
将你的测试和参考文件添加到代码库中
$ cd [path to repo]/test/contributors/ttwf/yourName/incoming
$ hg add ttwf-reftest-tutorial-001.html
$ hg add reference
$ hg commit -m "TTWF reftest tutorial"
$ hg push
merge
$ hg pull
$ hg merge
$ hg commit -m "Merge"
$ hg push
整合反馈意见并将测试移到submitted目录
cd [path to repo]/test/contributors/ttwf/yourName
$ hg pull -u
$ hg merge
$ hg commit -m "merging"
$ hg mv incoming/ttwf-reftest-tutorial-001.html submitted/ttwf-reftest-tutorial-001.html
$ hg mv incoming/reference/ttwf-reftest-tutorial-ref.html submitted/reference/ttwf-reftest-tutorial-ref.html
$ hg commit -m "moved the TTWF reftest tutorial to the submitted folder"
$ hg push
基本命令:
hg pull (gets the latest changes from the CSSWG Test repo)
To update after pull, use: hg pull -u (alleviates need to do hg update)
hg update (applies the latest changes pulled to your local repo)
hg status (displays list of locally changed files)
hg add (stages a new or modified local file for commit)
hg remove (stages the removal of a local file for commit)
hg merge (merges local changes with updates pulled from CSSWG Test repo)
hg commit (commits local changes to local repository)
To include commit message, use: hg commit -m "Commit message here"
hg push (pushes local changes to the CSSWG Test repository)
用户想更新本地的代码库:
hg pull -u (pulls and applies latest changes from CSSWG repo to local repo)
用户想推送本地的变化:
hg status (check which files are stages for commit)
hg add fileName (stages file for commit, repeat for each file)
hg status (confirm all files are stages for commit)
hg commit -m "Commit message here" (Commit to local repo)
hg push (pushes locally committed changes to CSSWG Test repo)
2012-10-21
ttwfbj@gmail.com
public-html-testsuite@w3.org