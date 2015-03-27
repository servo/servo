########################################################################
# $Id$
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
EXAMPLESDIR = .
# The settings of SRCDIR (where to find the xsl style sheets), OBJDIR (where to
# find widlproc and widlprocxml.dtd) and EXAMPLESOBJDIR (where to put the
# resulting html files and any intermediate files) reflect the directory
# structure of the released widlproc.zip file when unzipped. ../Makefile calls
# this Makefile with these settings overridden to reflect the directory
# structure in svn.
SRCDIR = ..
OBJDIR = ..
EXAMPLESOBJDIR = obj

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

WIDLS = $(patsubst $(EXAMPLESDIR)/%, %, $(wildcard $(EXAMPLESDIR)/*.widl))

test : $(patsubst %.widl, $(EXAMPLESOBJDIR)/%.html, $(WIDLS)) $(EXAMPLESOBJDIR)/widlhtml.css
	@echo "$@ pass"

$(EXAMPLESOBJDIR)/%.html : $(EXAMPLESOBJDIR)/%.widlprocxml $(SRCDIR)/widlprocxmltohtml.xsl Makefile
	cp $(SRCDIR)/widlprocxmltohtml.xsl $(dir $@)/
	xsltproc $(dir $@)/widlprocxmltohtml.xsl $< >$@

$(EXAMPLESOBJDIR)/%.widlprocxml : $(EXAMPLESDIR)/%.widl $(WIDLPROC) $(DTD) Makefile
	mkdir -p $(dir $@)
	$(WIDLPROC) $< >$@
	cp $(OBJDIR)/widlprocxml.dtd $(dir $@)/
	xmllint --noout --dtdvalid $(DTD) $@

$(EXAMPLESOBJDIR)/widlhtml.css : $(EXAMPLESDIR)/widlhtml.css
	cp $< $@

.DELETE_ON_ERROR:

