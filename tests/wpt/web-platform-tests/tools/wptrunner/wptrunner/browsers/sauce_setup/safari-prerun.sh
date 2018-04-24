#!/bin/bash
curl https://raw.githubusercontent.com/w3c/web-platform-tests/master/fonts/Ahem.ttf > ~/Library/Fonts/Ahem.ttf
defaults write com.apple.Safari com.apple.Safari.ContentPageGroupIdentifier.WebKit2JavaScriptCanOpenWindowsAutomatically -bool true
