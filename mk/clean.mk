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
	$(Q)cd $(B)/src/components/util/ && rm -rf libutil*.dylib libutil*.dSYM libutil*.so $(DONE_util)

clean-msg:
	@$(call E, "cleaning msg")
	$(Q)cd $(B)/src/components/msg/ && rm -rf libmsg*.dylib libmsg*.dSYM libmsg*.so $(DONE_msg)

clean-net:
	@$(call E, "cleaning net")
	$(Q)cd $(B)/src/components/net/ && rm -rf libnet*.dylib libnet*.dSYM libnet*.so $(DONE_net)

clean-gfx:
	@$(call E, "cleaning gfx")
	$(Q)cd $(B)/src/components/gfx/ && rm -rf libgfx*.dylib libgfx*.dSYM libgfx*.so $(DONE_gfx)

clean-script:
	@$(call E, "cleaning script")
	$(Q)cd $(B)/src/components/script/ && rm -rf libscript*.dylib libscript*.dSYM libscript*.so $(DONE_script)

clean-style:
	@$(call E, "cleaning style")
	$(Q)cd $(B)/src/components/style/ && rm -rf libstyle*.dylib libstyle*.dSYM libstyle*.so $(DONE_style)

clean-servo: clean-gfx clean-util clean-net clean-script clean-msg clean-style
	@$(call E, "cleaning servo")
	$(Q)rm -f servo servo-test $(foreach lib_crate,$(SERVO_LIB_CRATES),servo-test-$(lib_crate)) libservo*.so
	$(Q)cd $(BINDINGS_SRC) && rm -f *.pkl
