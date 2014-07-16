define DEF_SUBMODULE_CLEAN_RULES
# clean target
clean-$(1) : 
	@$$(call E, make clean: $(1))
	$$(Q)rm -f $$(DONE_$(1))
	$$(Q)$$(MAKE) -C $$(B)src/$$(PATH_$(1)) clean

# add these targets to meta-targets
DEPS_CLEAN_ALL += $(1)
endef

$(foreach submodule,$(SUBMODULES),\
$(eval $(call DEF_SUBMODULE_CLEAN_RULES,$(submodule))))

DEPS_CLEAN_TARGETS_ALL = $(addprefix clean-,$(DEPS_CLEAN_ALL))
DEPS_CLEAN_TARGETS_FAST = $(addprefix clean-,$(filter-out $(SLOW_BUILDS),$(DEPS_CLEAN_ALL)))

.PHONY:	clean $(DEPS_CLEAN_TARGETS_ALL)

clean: $(DEPS_CLEAN_TARGETS_ALL) clean-servo
	@$(call E, "cleaning:")
	@$(call E, "    $(DEPS_CLEAN_ALL)")

clean-fast: $(DEPS_CLEAN_TARGETS_FAST) clean-servo
	@$(call E, "cleaning:")
	@$(call E, "    $(filter-out $(SLOW_BUILDS),$(DEPS_CLEAN_ALL))")

clean-util:
	@$(call E, "cleaning util")
	$(Q)cd $(B)/src/components/util/ && rm -rf libutil*.dylib libutil*.rlib libutil*.dSYM libutil*.so $(DONE_util)

clean-embedding:
	@$(call E, "cleaning embedding")
	$(Q)cd $(B)/src/components/embedding/ && rm -rf libembedding*.dylib libembedding*.dSYM libembedding*.so $(DONE_embedding)

clean-msg:
	@$(call E, "cleaning msg")
	$(Q)cd $(B)/src/components/msg/ && rm -rf libmsg*.dylib libmsg*.rlib libmsg*.dSYM libmsg*.so $(DONE_msg)

clean-net:
	@$(call E, "cleaning net")
	$(Q)cd $(B)/src/components/net/ && rm -rf libnet*.dylib libnet*.rlib libnet*.dSYM libnet*.so $(DONE_net)

clean-gfx:
	@$(call E, "cleaning gfx")
	$(Q)cd $(B)/src/components/gfx/ && rm -rf libgfx*.dylib libgfx*.rlib libgfx*.dSYM libgfx*.so $(DONE_gfx)

clean-script:
	@$(call E, "cleaning script")
	$(Q)cd $(B)/src/components/script/ && rm -rf libscript*.dylib libscript*.rlib libscript*.dSYM libscript*.so $(DONE_script) && find $(S)/src/components/script/ -name \*.pyc -delete

clean-style:
	@$(call E, "cleaning style")
	$(Q)cd $(B)/src/components/style/ && rm -rf libstyle*.dylib libstyle*.rlib libstyle*.dSYM libstyle*.so $(DONE_style)

clean-wpt:
	$(Q)rm -r _virtualenv
	$(Q)rm $(S)/src/test/wpt/metadata/MANIFEST.json

clean-servo: clean-gfx clean-util clean-embedding clean-net clean-script clean-msg clean-style
	@$(call E, "cleaning servo")
	$(Q)rm -f servo servo-test $(foreach lib_crate,$(SERVO_LIB_CRATES),servo-test-$(lib_crate)) libservo*.so libservo*.a
	$(Q)cd $(BINDINGS_SRC) && rm -f *.pkl *.rs

clean-rust-snapshot-archives:
	@$(call E, "cleaning rust snapshot archives")
	$(Q)cd $(B)/rust_snapshot/ && rm -rf *.tgz