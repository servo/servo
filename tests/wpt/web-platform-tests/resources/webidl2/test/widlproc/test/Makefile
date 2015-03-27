########################################################################
# $Id: Makefile 418 2010-06-16 15:26:51Z tpr $
# Copyright 2009 Aplix Corporation. All rights reserved.
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#     http://www.apache.org/licenses/LICENSE-2.0
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
########################################################################

UNAME = $(shell uname)
VALIDTESTDIR = valid/idl
VALIDTESTREFDIR = valid/xml
VALIDTESTOBJDIR = valid/obj
INVALIDTESTDIR = invalid/idl
INVALIDTESTREFDIR = invalid/error
INVALIDTESTOBJDIR = invalid/obj

# The settings of SRCDIR (where to find the xsl style sheets), OBJDIR (where to
# find widlproc and widlprocxml.dtd) and VALDTESTOBJDIR (where to put the
# resulting XML files) reflect the directory
# structure of the released widlproc.zip file when unzipped. ../Makefile calls
# this Makefile with these settings overridden to reflect the directory
# structure in svn.
SRCDIR = ..
OBJDIR = ..

########################################################################
# Linux configuration
#
ifneq (,$(filter Linux%, $(UNAME))) 

EXESUFFIX =

else
########################################################################
# Darwin configuration
#
ifneq (,$(filter Darwin%, $(UNAME))) 

EXESUFFIX =

else
########################################################################
# Windows (cygwin but using MS compiler) configuration
#
ifneq (,$(filter CYGWIN%, $(UNAME))) 

EXESUFFIX = .exe

endif
endif
endif

########################################################################
# Common makefile
#
WIDLPROC = $(OBJDIR)/widlproc$(EXESUFFIX)
DTD = $(OBJDIR)/widlprocxml.dtd

VALID_WIDLS = $(patsubst $(VALIDTESTDIR)/%, %, $(wildcard $(VALIDTESTDIR)/*.widl))
INVALID_WIDLS = $(patsubst $(INVALIDTESTDIR)/%, %, $(wildcard $(INVALIDTESTDIR)/*.widl))

test : $(patsubst %.widl, $(VALIDTESTOBJDIR)/%.widlprocxml, $(VALID_WIDLS)) $(patsubst %.widl, $(INVALIDTESTOBJDIR)/%.txt, $(INVALID_WIDLS)) $(WIDLPROC) 
	@echo "$@ pass"

#$(EXAMPLESOBJDIR)/%.html : $(EXAMPLESOBJDIR)/%.widlprocxml $(SRCDIR)/widlprocxmltohtml.xsl Makefile
#	cp $(SRCDIR)/widlprocxmltohtml.xsl $(dir $@)/
#	xsltproc $(dir $@)/widlprocxmltohtml.xsl $< >$@


$(VALIDTESTOBJDIR)/%.widlprocxml : $(VALIDTESTDIR)/%.widl $(WIDLPROC) $(DTD) Makefile $(patsubst %.widl, $(VALIDTESTREFDIR)/%.widlprocxml, $(VALID_WIDLS))
	mkdir -p $(dir $@)
	$(WIDLPROC) $< >$@
	cp $(OBJDIR)/widlprocxml.dtd $(dir $@)/
	xmllint --noout --dtdvalid $(DTD) $@
	diff $@ $(VALIDTESTREFDIR)/`basename $@`

$(INVALIDTESTOBJDIR)/%.txt : $(INVALIDTESTDIR)/%.widl $(WIDLPROC) Makefile $(patsubst %.widl, $(INVALIDTESTREFDIR)/%.txt, $(INVALID_WIDLS))
	mkdir -p $(dir $@)
	-$(WIDLPROC) $<  >$@ 2>&1
	diff $@ $(INVALIDTESTREFDIR)/`basename $@`



.DELETE_ON_ERROR:

