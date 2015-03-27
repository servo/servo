#!/usr/bin/env python
"""Spider to try and find bugs in the parser. Requires httplib2 and elementtree

usage:
import spider
s = spider.Spider()
s.spider("http://www.google.com", maxURLs=100)
"""

import urllib.request, urllib.error, urllib.parse
import urllib.robotparser
import md5

import httplib2

import html5lib
from html5lib.treebuilders import etree

class Spider(object):
    def __init__(self):
        self.unvisitedURLs = set()
        self.visitedURLs = set()
        self.buggyURLs=set()
        self.robotParser = urllib.robotparser.RobotFileParser()
        self.contentDigest = {}
        self.http = httplib2.Http(".cache")

    def run(self, initialURL, maxURLs=1000):
        urlNumber = 0
        self.visitedURLs.add(initialURL)
        content = self.loadURL(initialURL)
        while maxURLs is None or urlNumber < maxURLs:
            if content is not None:
                self.parse(content)
                urlNumber += 1
            if not self.unvisitedURLs:
                break
            content = self.loadURL(self.unvisitedURLs.pop())

    def parse(self, content):
        failed = False
        p = html5lib.HTMLParser(tree=etree.TreeBuilder)
        try:
            tree = p.parse(content)
        except:
            self.buggyURLs.add(self.currentURL)
            failed = True
            print("BUGGY:", self.currentURL)
        self.visitedURLs.add(self.currentURL)
        if not failed:
            self.updateURLs(tree)

    def loadURL(self, url):
        resp, content = self.http.request(url, "GET")
        self.currentURL = url
        digest = md5.md5(content).hexdigest()
        if digest in self.contentDigest:
            content = None
            self.visitedURLs.add(url)
        else:
            self.contentDigest[digest] = url

        if resp['status'] != "200":
            content = None

        return content

    def updateURLs(self, tree):
        """Take all the links in the current document, extract the URLs and
        update the list of visited and unvisited URLs according to whether we
        have seen them before or not"""
        urls = set()
        #Remove all links we have already visited
        for link in tree.findall(".//a"):
                try:
                    url = urllib.parse.urldefrag(link.attrib['href'])[0]
                    if (url and url not in self.unvisitedURLs and url
                        not in self.visitedURLs):
                        urls.add(url)
                except KeyError:
                    pass

        #Remove all non-http URLs and a dd a sutiable base URL where that is
        #missing
        newUrls = set()
        for url in urls:
            splitURL = list(urllib.parse.urlsplit(url))
            if splitURL[0] != "http":
                continue
            if splitURL[1] == "":
                splitURL[1] = urllib.parse.urlsplit(self.currentURL)[1]
            newUrls.add(urllib.parse.urlunsplit(splitURL))
        urls = newUrls

        responseHeaders = {}
        #Now we want to find the content types of the links we haven't visited
        for url in urls:
            try:
                resp, content = self.http.request(url, "HEAD")
                responseHeaders[url] = resp
            except AttributeError as KeyError:
                #Don't know why this happens
                pass


        #Remove links not of content-type html or pages not found
        #XXX - need to deal with other status codes?
        toVisit = set([url for url in urls if url in responseHeaders and
                      "html" in responseHeaders[url]['content-type'] and
                      responseHeaders[url]['status'] == "200"])

        #Now check we are allowed to spider the page
        for url in toVisit:
            robotURL = list(urllib.parse.urlsplit(url)[:2])
            robotURL.extend(["robots.txt", "", ""])
            robotURL = urllib.parse.urlunsplit(robotURL)
            self.robotParser.set_url(robotURL)
            if not self.robotParser.can_fetch("*", url):
                toVisit.remove(url)

        self.visitedURLs.update(urls)
        self.unvisitedURLs.update(toVisit)
