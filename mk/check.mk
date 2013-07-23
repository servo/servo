define DEF_SUBMODULE_TEST_RULES
# check target
.PHONY: check-$(1)
check-$(1) : $$(DONE_$(1))
	@$$(call E, check: $(1))

	$$(Q) \
	$$(ENV_CFLAGS_$(1)) \
	$$(ENV_RFLAGS_$(1)) \
	$$(MAKE) -C $$(B)src/$$(PATH_$(1)) check

DEPS_CHECK_ALL += $(1)
endef

$(foreach submodule,$(SUBMODULES),\
$(eval $(call DEF_SUBMODULE_TEST_RULES,$(submodule))))


# Testing targets

servo-test: $(DEPS_servo)
	@$(call E, check: servo)
	$(Q)$(RUSTC) $(RFLAGS_servo) --test -o $@ $<

reftest: $(S)src/test/harness/reftest/reftest.rs servo
	@$(call E, compile: $@)
	$(Q)$(RUSTC) -o $@ $<

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

.PHONY: check
check: $(DEPS_CHECK_TARGETS_FAST) check-servo tidy
	@$(call E, check: all)

.PHONY: check-all
check-all: $(DEPS_CHECK_TARGETS_ALL) check-servo tidy
	@$(call E, check: all)

.PHONY: check-servo
check-servo: servo-test
	@$(call E, check: servo)
	$(Q)./servo-test

.PHONY: check-ref
check-ref: reftest
	@$(call E, check: reftests)
	$(Q)./reftest $(S)src/test/ref/*.list

.PHONY: check-content
check-content: contenttest
	@$(call E, check: contenttests)
	$(Q)./contenttest --source-dir=$(S)src/test/html/content $(TESTNAME)

.PHONY: tidy
tidy:
	@$(call E, check: tidy)
	$(Q)python $(S)src/etc/tidy.py $(S)src
