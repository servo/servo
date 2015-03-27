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
INCDIRS = $(OBJDIR)
SRCDIR = src
DOCDIR = doc
EXAMPLESDIR = examples
OBJDIR = obj

########################################################################
# Linux configuration
#
ifneq (,$(filter Linux%, $(UNAME))) 

CFLAGS = -g -Wall -Werror -O0 $(patsubst %, -I%, $(INCDIRS))
OBJSUFFIX = .o
EXESUFFIX =
#LIBS = -lefence
OBJOPTION = -o
EXEOPTION = -o

else
########################################################################
# Darwin configuration
#
ifneq (,$(filter Darwin%, $(UNAME))) 

CFLAGS = -g -Wall -Werror -O2 $(patsubst %, -I%, $(INCDIRS))
OBJSUFFIX = .o
EXESUFFIX =
OBJOPTION = -o
# The -o in the following line has a space after it, which must not be removed.
EXEOPTION = -o 

else
########################################################################
# Windows (cygwin but using MS compiler) configuration
#
# this is messy - should probably use vcvars.bat
ifneq (,$(filter CYGWIN%, $(UNAME))) 
VISUALSTUDIODIR = $(wildcard /cygdrive/c/Program*Files/Microsoft*Visual*Studio*8)
SDKDIR = $(wildcard /cygdrive/c/Program*Files/Microsoft*SDKs/Windows/*/Lib)
ifeq (,$(VISUALSTUDIODIR))
VISUALSTUDIODIR = $(wildcard /cygdrive/c/Program\ Files\ */Microsoft*Visual*Studio*10*)
endif
ifeq (,$(VISUALSTUDIODIR))
VISUALSTUDIODIR = $(wildcard /cygdrive/c/Program\ Files\ */Microsoft*Visual*Studio*11*)
endif
# this is revelvant for vs2012 and windows 8 - sdk location has changed
ifeq (,$(SDKDIR))
SDKDIR = $(wildcard /cygdrive/c/Program\ Files\ */Windows*Kits)
endif

ifeq (,$(VISUALSTUDIODIR))
$(error Could not find  MS Visual Studio) 
else
WINVISUALSTUDIODIR = $(shell cygpath -w '$(VISUALSTUDIODIR)')
WINSDKDIR = $(shell cygpath -w '$(SDKDIR)')

#$(error $(VISUALSTUDIODIR)) 

CC = \
	Lib='$(WINVISUALSTUDIODIR)\VC\LIB;$(WINVISUALSTUDIODIR)\VC\PlatformSDK\Lib;$(WINSDKDIR)' \
	PATH='$(VISUALSTUDIODIR)/Common7/IDE:$(VISUALSTUDIODIR)/VC/BIN:$(VISUALSTUDIODIR)/Common7/Tools:$(VISUALSTUDIODIR)/SDK/v2.0/bin:$(VISUALSTUDIODIR)/8.0/Lib/win8/um/x86:'$$PATH \
	Include='$(WINVISUALSTUDIODIR)\VC\INCLUDE;$(WINVISUALSTUDIODIR)\VC\PlatformSDK\Include' \
	cl
endif

CFLAGS = /nologo /WX /W3 /wd4996 /Zi /O2 $(patsubst %, /I%, $(INCDIRS))
OBJSUFFIX = .obj
EXESUFFIX = .exe
OBJOPTION = /Fo
EXEOPTION = /Fe

endif
endif
endif

########################################################################
# Common makefile
#
WIDLPROC = $(OBJDIR)/widlproc$(EXESUFFIX)
DTD = $(OBJDIR)/widlprocxml.dtd

ALL = $(WIDLPROC) $(DTD)
all : $(ALL)

SRCS = \
	comment.c \
	lex.c \
	main.c \
	misc.c \
	node.c \
	parse.c \
	process.c

OBJS = $(patsubst %.c, $(OBJDIR)/%$(OBJSUFFIX), $(SRCS))
$(WIDLPROC) : $(OBJS)
	$(CC) $(CFLAGS) $(EXEOPTION)$@ $^ $(LIBS)

$(OBJDIR)/%$(OBJSUFFIX) : $(SRCDIR)/%.c
	mkdir -p $(dir $@)
	$(CC) $(CFLAGS) $(OBJOPTION)$@ -c $<

$(OBJDIR)/%.d : $(SRCDIR)/%.c
	mkdir -p $(dir $@)
	cc $(patsubst %, -I%, $(INCDIRS)) -MM -MG -MT $(patsubst %.d, %$(OBJSUFFIX), $@) $< | sed '$(patsubst %, s| \(%\)| $(OBJDIR)/\1|;, $(AUTOGENHEADERS))' >$@

include $(patsubst %.c, $(OBJDIR)/%.d, $(SRCS))


$(DTD) : $(DOCDIR)/htmltodtd.xsl $(DOCDIR)/widlproc.html
	xsltproc -html $^ >$@

clean :
	rm -f $(ALL) $(OBJS)

veryclean :
	rm -rf $(OBJDIR)

SVNFILES = $(shell test -d .svn && svn info -R . | sed -n 's/^Path: \(.*\)$$/\1/p')
SVNBRANCH = $(shell test -d .svn && svn info . | sed -n 's|^URL:.*/\([^/]*\)$$|\1|p')
SVNREV = $(shell test -d .svn && svn info -R . | sed -n 's/^Last Changed Rev: \([0-9][0-9]*\)$$/\1/p' | sort -g | tail -1)

SVNLOG = history
$(SVNLOG) : $(SVNFILES)
	svn log -vrHEAD:311 >$@

zip : $(OBJDIR)/widlproc-$(SVNBRANCH)$(SVNREV).zip
$(OBJDIR)/widlproc-$(SVNBRANCH)$(SVNREV).zip : $(WIDLPROC) $(DTD) $(DOCDIR)/widlproc.html $(SRCDIR)/widlprocxmltohtml.xsl Makefile $(SVNLOG)
	rm -f $@
	zip -j $@ $^ -x Makefile
	zip $@ examples/*.widl examples/*.css examples/Makefile examples/README examples/*.xsl examples/*.html

srczip : widlproc-src-$(SVNBRANCH)$(SVNREV).zip

widlproc-src-%.zip : $(SVNFILES) $(SVNLOG)
	zip $@ $^ 

examples :
	$(MAKE) -C examples SRCDIR=../src OBJDIR=../obj EXAMPLESOBJDIR=../obj/examples

test : $(OBJS)
	$(MAKE) -C test SRCDIR=../src OBJDIR=../obj

.DELETE_ON_ERROR:
