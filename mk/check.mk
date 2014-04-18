define DEF_SUBMODULE_TEST_RULES
# check target
.PHONY: check-$(1)
check-$(1) : $$(DONE_$(1))
	@$$(call E, check: $(1))

	$$(Q) \
	$$(ENV_CFLAGS_$(1)) \
	$$(ENV_CXXFLAGS_$(1)) \
	$$(ENV_RFLAGS_$(1)) \
	$$(MAKE) -C $$(B)src/$$(PATH_$(1)) check

DEPS_CHECK_ALL += $(1)
endef

$(foreach submodule,$(SUBMODULES),\
$(eval $(call DEF_SUBMODULE_TEST_RULES,$(submodule))))


define DEF_LIB_CRATE_TEST_RULES
servo-test-$(1): $$(DEPS_$(1))
	@$$(call E, compile: servo-test-$(1))
	$$(Q)$$(RUSTC) $$(RFLAGS_$(1)) --test -o $$@ $$<

.PHONY: check-servo-$(1)
check-servo-$(1): servo-test-$(1)
	@$$(call E, check: $(1))
	$$(Q)./servo-test-$(1)
endef

$(foreach lib_crate,$(SERVO_LIB_CRATES),\
$(eval $(call DEF_LIB_CRATE_TEST_RULES,$(lib_crate))))


# Testing targets

servo-test: $(DEPS_servo)
	@$(call E, check: servo)
	$(Q)$(RUSTC) $(RFLAGS_servo) --test -o $@ $<

reftest: $(S)src/test/harness/reftest/reftest.rs servo
	@$(call E, compile: $@)
	$(Q)$(RUSTC) -L$(B)/src/support/png/rust-png/ -L$(B)/src/support/png/libpng/ -o $@ $<

contenttest: $(S)src/test/harness/contenttest/contenttest.rs servo
	@$(call E, compile: $@)
	$(Q)$(RUSTC) $(RFLAGS_servo) -o $@ $< -L .


DEPS_CHECK_TESTABLE = $(filter-out $(NO_TESTS),$(DEPS_CHECK_ALL))
DEPS_CHECK_TARGETS_ALL = $(addprefix check-,$(DEPS_CHECK_TESTABLE))
DEPS_CHECK_TARGETS_FAST = $(addprefix check-,$(filter-out $(SLOW_TESTS),$(DEPS_CHECK_TESTABLE)))

.PHONY: check-test
check-test:
	@$(call E, check:)
	@$(call E, "    $(DEPS_CHECK_TARGETS_ALL)")

ifeq ($(CFG_OSTYPE),apple-darwin)
.PHONY: check
check: $(DEPS_CHECK_TARGETS_FAST) check-servo check-content check-ref tidy
	@$(call E, check: all)

.PHONY: check-all
check-all: $(DEPS_CHECK_TARGETS_ALL) check-servo check-content check-ref tidy
	@$(call E, check: all)
else
.PHONY: check
check: $(DEPS_CHECK_TARGETS_FAST) check-servo tidy
	@$(call E, check: all)

.PHONY: check-all
check-all: $(DEPS_CHECK_TARGETS_ALL) check-servo tidy
	@$(call E, check: all)
endif

.PHONY: check-servo
check-servo: $(foreach lib_crate,$(SERVO_LIB_CRATES),check-servo-$(lib_crate)) servo-test
	@$(call E, check: servo)
	$(Q)./servo-test

.PHONY: check-ref
check-ref: reftest
	@$(call E, check: reftests with GPU rendering)
	$(Q)./reftest $(S)src/test/ref/*.list
	@$(call E, check: reftests with CPU rendering)
	$(Q)./reftest $(S)src/test/ref/*.list -- -c

.PHONY: check-content
check-content: contenttest
	@$(call E, check: contenttests)
	$(Q)./contenttest --source-dir=$(S)src/test/content $(TESTNAME)

.PHONY: check-wpt
check-wpt:
	bash $(S)src/test/wpt/run.sh $(S) $(B)

.PHONY: tidy
tidy:
	@$(call E, check: tidy)
	$(Q)python $(S)src/etc/tidy.py $(S)src
