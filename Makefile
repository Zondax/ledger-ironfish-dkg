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



zemu_install_ironfish_link:
	cd ironfish && yarn unlink || true
	cd $(TESTS_ZEMU_DIR) && yarn unlink @ironfish/rust-nodejs || true
	# Now build and link
	cd ironfish && yarn install && cd ironfish-rust-nodejs && yarn link || true
	cd $(TESTS_ZEMU_DIR) && yarn link @ironfish/rust-nodejs || true

# Redefine zemu_install, as we still need to compile the ironfish package locally
.PHONY: zemu_install
zemu_install: zemu_install_ironfish_link zemu_install_js_link
	# and now install everything
	cd $(TESTS_ZEMU_DIR) && yarn install