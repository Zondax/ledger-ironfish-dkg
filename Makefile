TESTS_JS_PACKAGE = "@zondax/ledger-ironfish"
TESTS_JS_DIR = $(CURDIR)/js


ifeq ($(BOLOS_SDK),)
# In this case, there is not predefined SDK and we run dockerized
# When not using the SDK, we override and build the XL complete app
PRODUCTION_BUILD ?= 1
include $(CURDIR)/deps/ledger-zxlib/dockerized_build.mk

else
default:
	$(MAKE) -C app
%:
	$(info "Calling app Makefile for target $@")
	COIN=$(COIN) $(MAKE) -C app $@
endif

# Copy files quickly from a source to a folder with a regular expresions
# In some cases, when something change in the screens for some command, it changes for each participant.
# This makes it hard and tedious to update each participant snapshots (one per test run).
# This allows to quickly update the other images from the first run
.PHONY: copy-files
copy-files:
	find . -type d -name "*sp-*" -exec cp ../snapshots-tmp/sp-dkg-p2-m2-0-identity/00003.png {} \;

.PHONY: rust_tests
rust_tests:
	cd app && cargo +stable test --no-default-features
