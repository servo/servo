#!/usr/bin/env python

# CSS Test Suite Build Script
# Copyright 2011 Hewlett-Packard Development Company, L.P.
# Initial code by fantasai, joint copyright 2010 W3C and Microsoft
# Licensed under BSD 3-Clause: <http://www.w3.org/Consortium/Legal/2008/03-bsd-license>

import sys
import os
import json
import optparse
import shutil
from collections import defaultdict
from apiclient import apiclient
from w3ctestlib import Sources, Utils, Suite, Indexer
from mercurial import ui



class Builder(object):
    def __init__(self, ui, outputPath, backupPath, ignorePaths, onlyCache):
        self.reset(onlyCache)
        self.ui = ui
        self.skipDirs = ('support')
        self.rawDirs = {'other-formats': 'other'}
        self.sourceTree = Sources.SourceTree()
        self.sourceCache = Sources.SourceCache(self.sourceTree)
        self.cacheDir = 'tools/cache'
        self.outputPath = outputPath.rstrip('/') if (outputPath) else 'dist'
        self.backupPath = backupPath.rstrip('/') if (backupPath) else None
        self.ignorePaths = [path.rstrip('/') for path in ignorePaths] if (ignorePaths) else []
        self.workPath = 'build-temp'
        self.ignorePaths += (self.outputPath, self.backupPath, self.workPath)

    def reset(self, onlyCache):
        self.useCacheOnly = onlyCache
        self.shepherd = apiclient.apiclient.APIClient('https://api.csswg.org/shepherd/', version = 'vnd.csswg.shepherd.v1') if (not onlyCache) else None
        self.cacheData = False
        self.testSuiteData = {}
        self.specificationData = {}
        self.flagData = {}
        self.specNames = {}
        self.specAnchors = {}
        self.buildSuiteNames = []
        self.buildSpecNames = {}
        self.testSuites = {}


    def _loadShepherdData(self, apiName, description, **kwargs):
        self.ui.status("Loading ", description, " information\n")
        cacheFile = os.path.join(self.cacheDir, apiName + '.json')
        if (not self.useCacheOnly or (not os.path.exists(cacheFile))):
            result = self.shepherd.get(apiName, **kwargs)
            if (result and (200 == result.status)):
                data = {}
                for name in result.data:    # trim leading _
                    data[name[1:]] = result.data[name]
                with open(cacheFile, 'w') as file:
                    json.dump(data, file)
                return data
            self.ui.status("Shepherd API call failed, result: ", result.status if result else 'None', "\n")
        if (os.path.exists(cacheFile)):
            self.ui.status("Loading cached data.\n")
            try:
                with open(cacheFile, 'r') as file:
                    return json.load(file)
            except:
                pass
        return None

    def _addAnchors(self, anchors, specName):
        for anchor in anchors:
            self.specAnchors[specName].add(anchor['uri'].lower())
            if ('children' in anchor):
                self._addAnchors(anchor['children'], specName)

    def _normalizeScheme(self, url):
        if (url and url.startswith('http:')):
            return 'https:' + url[5:]
        return url

    def getSpecName(self, url):
        if (not self.specNames):
            for specName in self.specificationData:
                specData = self.specificationData[specName]
                officialURL = self._normalizeScheme(specData.get('base_uri'))
                if (officialURL):
                    if (officialURL.endswith('/')):
                        officialURL = officialURL[:-1]
                    self.specNames[officialURL.lower()] = specName
                draftURL = self._normalizeScheme(specData.get('draft_uri'))
                if (draftURL):
                    if (draftURL.endswith('/')):
                        draftURL = draftURL[:-1]
                    self.specNames[draftURL.lower()] = specName
                self.specAnchors[specName] = set()
                if ('anchors' in specData):
                    self._addAnchors(specData['anchors'], specName)
                if ('draft_anchors' in specData):
                    self._addAnchors(specData['draft_anchors'], specName)

        url = self._normalizeScheme(url.lower())
        for specURL in self.specNames:
            if (url.startswith(specURL) and
                ((url == specURL) or
                 url.startswith(specURL + '/') or
                 url.startswith(specURL + '#'))):
                anchorURL = url[len(specURL):]
                if (anchorURL.startswith('/')):
                    anchorURL = anchorURL[1:]
                specName = self.specNames[specURL]
                if (anchorURL in self.specAnchors[specName]):
                    return (specName, anchorURL)
                return (specName, None)
        return (None, None)

    def gatherTests(self, dir):
        dirName = os.path.basename(dir)
        if (dirName in self.skipDirs):
            return

        self.ui.note("Scanning directory: ", dir, "\n")
        suiteFileNames = defaultdict(set)
        for fileName in Utils.listfiles(dir):
            filePath = os.path.join(dir, fileName)
            if not self.sourceTree.isTestCase(filePath):
                continue

            source = self.sourceCache.generateSource(filePath, fileName)
            if not source.isTest():
                continue

            metaData = source.getMetadata(True)
            if not metaData:
                if (source.errors):
                    self.ui.warn("Error parsing '", filePath, "': ", ' '.join(source.errors), "\n")
                else:
                    self.ui.warn("No metadata available for '", filePath, "'\n")
                continue

            for specURL in metaData['links']:
                specName, anchorURL = self.getSpecName(specURL)
                if not specName:
                    self.ui.note("Unknown specification URL: ", specURL, "\n  in: ", filePath, "\n")
                    continue

                if not specName in self.buildSpecNames:
                    continue

                if not anchorURL:
                    self.ui.warn("Test links to unknown specification anchor: ", specURL, "\n  in: ", filePath, "\n")
                    continue

                for testSuiteName in self.buildSpecNames[specName]:
                    formats = self.testSuiteData[testSuiteName].get('formats')
                    if (formats):
                        for formatName in formats:
                            if (((formatName) in self.formatData) and
                                (self.formatData[formatName].get('mime_type') == source.mimetype)):
                                suiteFileNames[testSuiteName].add(fileName)
                                break
                        else:
                            self.ui.note("Test not in acceptable format: ", source.mimetype, "\n for: ", filePath, "\n")

        for testSuiteName in suiteFileNames:
            if (dirName in self.rawDirs):
                self.testSuites[testSuiteName].addRaw(dir, self.rawDirs[dirName])
            else:
                self.testSuites[testSuiteName].addTestsByList(dir, suiteFileNames[testSuiteName])

        for subDir in Utils.listdirs(dir):
            subDirPath = os.path.join(dir, subDir)
            if (not (self.sourceTree.isIgnoredDir(subDirPath) or (subDirPath in self.ignorePaths))):
                self.gatherTests(subDirPath)


    def _findSections(self, baseURL, anchors, sectionData, parentSectionName = ''):
        if (anchors):
            for anchor in anchors:
                if ('section' in anchor):
                    sectionData.append((baseURL + anchor['uri'], anchor['name'],
                                        anchor['title'] if 'title' in anchor else 'Untitled'))
                else:
                    sectionData.append((baseURL + anchor['uri'], parentSectionName + '.#' + anchor['name'], None))
                if ('children' in anchor):
                    self._findSections(baseURL, anchor['children'], sectionData, anchor['name'])
        return sectionData

    def getSections(self, specName):
        specData = self.specificationData[specName]
        specURL = specData['base_uri'] if ('base_uri' in specData) else specData.get('draft_uri')
        anchorData = specData['anchors'] if ('anchors' in specData) else specData['draft_anchors']
        sectionData = []
        self._findSections(specURL, anchorData, sectionData)
        return sectionData

    def _user(self, user):
        if (user):
            data = user['full_name']
            if ('organization' in user):
                data += ', ' + user['organization']
            if ('uri' in user):
                data += ', ' + user['uri']
            elif ('email' in user):
                data += ', &lt;' + user['email'].replace('@', ' @') + '&gt;'
            return data
        return 'None Yet'

    def getSuiteData(self):
        data = {}
        for testSuiteName in self.testSuiteData:
            testSuiteData = self.testSuiteData[testSuiteName]
            specData = self.specificationData[testSuiteData['specs'][0]]
            data[testSuiteName] = {
                'title': testSuiteData['title'] if ('title' in testSuiteData) else 'Untitled',
                'spec': specData['title'] if ('title' in specData) else specData['name'],
                'specroot': specData['base_uri'] if ('base_uri' in specData) else specData.get('draft_uri'),
                'draftroot': specData['draft_uri'] if ('draft_uri' in specData) else specData.get('base_uri'),
                'owner': self._user(testSuiteData['owners'][0] if ('owners' in testSuiteData) else None),
                'harness': testSuiteName,
                'status': testSuiteData['status'] if ('status' in testSuiteData) else 'Unknown'
            }
        return data

    def getFlags(self):
        data = {}
        for flag in self.flagData:
            flagData = self.flagData[flag]
            data[flag] = {
                'title': flagData['description'] if ('description' in flagData) else 'Unknown',
                'abbr': flagData['title'] if ('title' in flagData) else flag
            }
        return data


    def build(self, testSuiteNames):
        try:
            os.makedirs(self.cacheDir)
        except:
            pass
        self.testSuiteData = self._loadShepherdData('test_suites', 'test suite', repo = 'css')
        if (not self.testSuiteData):
            self.ui.warn("ERROR: Unable to load test suite information.\n")
            return -1
        if (testSuiteNames):
            self.buildSuiteNames = []
            for testSuiteName in testSuiteNames:
                if (testSuiteName in self.testSuiteData):
                    self.buildSuiteNames.append(testSuiteName)
                else:
                    self.ui.status("Unknown test suite: ", testSuiteName, "\n")
        else:
            self.buildSuiteNames = [testSuiteName for testSuiteName in self.testSuiteData if self.testSuiteData[testSuiteName].get('build')]

        self.buildSpecNames = defaultdict(list)
        if (self.buildSuiteNames):
            self.specificationData = self._loadShepherdData('specifications', 'specification', anchors = True, draft = True)
            if (not self.specificationData):
                self.ui.warn("ERROR: Unable to load specification information.\n")
                return -2
            for testSuiteName in self.buildSuiteNames:
                specNames = self.testSuiteData[testSuiteName].get('specs')
                if (specNames):
                    for specName in specNames:
                        if (specName in self.specificationData):
                            self.buildSpecNames[specName].append(testSuiteName)
                        else:
                            self.ui.warn("WARNING: Test suite '", testSuiteName, "' references unknown specification: '", specName, "'.\n")
                else:
                    self.ui.warn("ERROR: Test suite '", testSuiteName, "' does not have target specifications.\n")
                    return -6
        else:
            self.ui.status("No test suites identified\n")
            return 0

        if (not self.buildSpecNames):
            self.ui.status("No target specifications identified\n")
            return -3

        self.flagData = self._loadShepherdData('test_flags', 'test flag')
        if (not self.flagData):
            self.ui.warn("ERROR: Unable to load flag information\n")
            return -4

        self.formatData = self._loadShepherdData('test_formats', 'test format')
        if (not self.formatData):
            self.ui.warn("ERROR: Unable to load format information\n")
            return -5


        self.buildSuiteNames.sort()

        for testSuiteName in self.buildSuiteNames:
            data = self.testSuiteData[testSuiteName]
            specData = self.specificationData[data['specs'][0]]
            specURL = specData['base_uri'] if ('base_uri' in specData) else specData.get('draft_uri')
            draftURL = specData['draft_uri'] if ('draft_uri' in specData) else specData.get('base_uri')
            self.testSuites[testSuiteName] = Suite.TestSuite(testSuiteName, data['title'], specURL, draftURL, self.sourceCache, self.ui) # XXX need to support multiple specs
            if ('formats' in data):
                self.testSuites[testSuiteName].setFormats(data['formats'])

        self.ui.status("Scanning test files\n")

        for dir in Utils.listdirs('.'):
            if (not (self.sourceTree.isIgnoredDir(dir) or (dir in self.ignorePaths))):
                self.gatherTests(dir)


        if (os.path.exists(self.workPath)):
            self.ui.note("Clearing work path: ", self.workPath, "\n")
            shutil.rmtree(self.workPath)

        suiteData = self.getSuiteData()
        flagData = self.getFlags()
        templatePath = os.path.join('tools', 'templates')
        for testSuiteName in self.buildSuiteNames:
            testSuite = self.testSuites[testSuiteName]
            self.ui.status("Building ", testSuiteName, "\n")
            specSections = self.getSections(self.testSuiteData[testSuiteName]['specs'][0])
            indexer = Indexer.Indexer(testSuite, specSections, suiteData, flagData, True,
                                      templatePathList = [templatePath],
                                      extraData = {'devel' : False, 'official' : True })
            workPath = os.path.join(self.workPath, testSuiteName)
            testSuite.buildInto(workPath, indexer)

        # move from work path to output path
        for testSuiteName in self.buildSuiteNames:
            workPath = os.path.join(self.workPath, testSuiteName)
            outputPath = os.path.join(self.outputPath, testSuiteName)
            backupPath = os.path.join(self.backupPath, testSuiteName) if (self.backupPath) else None
            if (os.path.exists(workPath)):
                if (os.path.exists(outputPath)):
                    if (backupPath):
                        if (os.path.exists(backupPath)):
                            self.ui.note("Removing ", backupPath, "\n")
                            shutil.rmtree(backupPath)       # delete old backup
                        self.ui.note("Backing up ", outputPath, " to ", backupPath, "\n")
                        shutil.move(outputPath, backupPath) # backup old output
                    else:
                        self.ui.note("Removing ", outputPath, "\n")
                        shutil.rmtree(outputPath)           # no backups, delete old output
                self.ui.note("Moving ", workPath, " to ", outputPath, "\n")
                shutil.move(workPath, outputPath)
        if (os.path.exists(self.workPath)):
            shutil.rmtree(self.workPath)
        return 0



if __name__ == "__main__":      # called from the command line
    parser = optparse.OptionParser(usage = "usage: %prog [options] test_suite [...]")
    parser.add_option("-q", "--quiet",
                      action = "store_true", dest = "quiet", default = False,
                      help = "don't print status messages to stdout")
    parser.add_option("-d", "--debug",
                      action = "store_true", dest = "debug", default = False,
                      help = "print detailed debugging information to stdout")
    parser.add_option("-v", "--verbose",
                      action = "store_true", dest = "verbose", default = False,
                      help = "print more detailed debugging information to stdout")
    parser.add_option("-o", "--output", dest = "output", metavar = "OUTPUT_PATH",
                      help = "Path to build into (default 'dist')")
    parser.add_option("-b", "--backup", dest = "backup", metavar = "BACKUP_PATH",
                      help = "Path to preserve old version to")
    parser.add_option("-i", "--ignore",
                      action = "append", dest = "ignore", metavar = "IGNORE_PATH",
                      help = "Ignore files in this path")
    parser.add_option("-c", "--cache",
                      action = "store_true", dest = "cache", default = False,
                      help = "use cached test suite and specification data only")
    (options, args) = parser.parse_args()


    ui = ui.ui()
    ui.setconfig('ui', 'debug', str(options.debug))
    ui.setconfig('ui', 'quiet', str(options.quiet))
    ui.setconfig('ui', 'verbose', str(options.verbose))

    builder = Builder(ui, options.output, options.backup, options.ignore, options.cache)
    result = builder.build(args)
    quit(result)
