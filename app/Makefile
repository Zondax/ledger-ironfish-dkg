#*******************************************************************************
#  Ledger App
#  (c) 2018 - 2024 Zondax AG
#  (c) 2017 Ledger
#
#  Licensed under the Apache License, Version 2.0 (the "License");
#  you may not use this file except in compliance with the License.
#  You may obtain a copy of the License at
#
#      http://www.apache.org/licenses/LICENSE-2.0
#
#  Unless required by applicable law or agreed to in writing, software
#  distributed under the License is distributed on an "AS IS" BASIS,
#  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
#  See the License for the specific language governing permissions and
#  limitations under the License.
#*******************************************************************************

MY_DIR := $(dir $(lastword $(MAKEFILE_LIST)))

include $(CURDIR)/../deps/ledger-zxlib/makefiles/Makefile.installer_script

# Set the default value for PRODUCTION_BUILD to 0 if not already defined
PRODUCTION_BUILD ?= 1

$(info ************ TARGET_NAME  = [$(TARGET_NAME)])

# Display whether this is a production build or for internal use
ifeq ($(PRODUCTION_BUILD), 1)
    $(info ************ PRODUCTION_BUILD  = [PRODUCTION BUILD])
else
    $(info ************ PRODUCTION_BUILD  = [INTERNAL USE])
endif

# Display whether swap functionality is enabled or not
ifdef HAVE_SWAP
    $(info ************ HAVE_SWAP  = [ENABLED])
    DEFINES += HAVE_SWAP=$(HAVE_SWAP)
else
    $(info ************ HAVE_SWAP  = [DISABLED])
endif

include $(CURDIR)/../deps/ledger-zxlib/makefiles/Makefile.app_testing

ifndef COIN
COIN=IRON_DKG
endif

$(info COIN  = [$(COIN)])

ifeq ($(COIN),IRON_DKG)
# Main app configuration
APPNAME = "Ironfish DKG"
APPPATH = "44'/1338'"

else
define error_message

COIN value not supported: [$(COIN)]

endef
$(error "$(error_message)")
endif

$(info PATHS LIST = $(APPPATH))

APP_LOAD_PARAMS = --curve ed25519 --appName $(APPNAME)
APP_DELETE_PARAMS = --appName $(APPNAME)

include $(CURDIR)/../deps/ledger-zxlib/makefiles/Makefile.devices

DEFAULT_ICON = nanos_$(ICON_NAME).gif

$(info TARGET_NAME  = [$(TARGET_NAME)])
$(info ICONNAME  = [$(ICONNAME)])

ifndef ICONNAME
$(error ICONNAME is not set)
endif


include $(CURDIR)/../deps/ledger-zxlib/makefiles/Makefile.side_loading

.PHONY: buildX
buildX:
	BOLOS_SDK= PRODUCTION_BUILD=$(PRODUCTION_BUILD) cargo ledger build nanox
	FOLDER=nanox make post-build

.PHONY: buildS2
buildS2:
	BOLOS_SDK= PRODUCTION_BUILD=$(PRODUCTION_BUILD) cargo ledger build nanosplus
	FOLDER=nanosplus make post-build

.PHONY: buildST
buildST:
	BOLOS_SDK= PRODUCTION_BUILD=$(PRODUCTION_BUILD) cargo ledger build stax
	FOLDER=stax make post-build


.PHONY: buildFL
buildFL:
	BOLOS_SDK= PRODUCTION_BUILD=$(PRODUCTION_BUILD) cargo ledger build flex
	FOLDER=flex make post-build


.PHONY: clean_output
clean_output:
	@echo "Removing output files"
	@rm -f output/app* || true

.PHONY: clean_build
clean_build:
	cargo clean

.PHONY: clean
clean: clean_output clean_build

.PHONY: listvariants
listvariants:
	@echo VARIANTS COIN ZEC

.PHONY: version
version:
	@echo "v$(APPVERSION)" > app.version

.PHONY: format
format:
	cargo fmt --all

.PHONY: lint-nanosplus
lint-nanosplus:
	APPVERSION=$(APPVERSION) cargo clippy --target nanosplus -- -Dwarnings

.PHONY: lint-nanox
lint-nanox:
	APPVERSION=$(APPVERSION) cargo clippy --target nanox -- -Dwarnings

.PHONY: lint-flex
lint-flex:
	APPVERSION=$(APPVERSION) cargo clippy --target flex -- -Dwarnings

.PHONY: lint-stax
lint-stax:
	APPVERSION=$(APPVERSION) cargo clippy --target stax -- -Dwarnings


.PHONY: lint-nanosplus-fix
lint-nanosplus-fix:
	APPVERSION=$(APPVERSION) cargo clippy --target nanosplus --fix

.PHONY: lint-nanox-fix
lint-nanox-fix:
	APPVERSION=$(APPVERSION) cargo clippy --target nanox --fix

.PHONY: lint-flex-fix
lint-flex-fix:
	APPVERSION=$(APPVERSION) cargo clippy --target flex --fix

.PHONY: lint-stax-fix
lint-stax-fix:
	APPVERSION=$(APPVERSION) cargo clippy --target stax --fix

.PHONY: post-build
post-build: move-files-to-output read-params-from-json get-icon-hex create-install-script

.PHONY: move-files-to-output
move-files-to-output:
	cp target/$(FOLDER)/release/ironfish-dkg bin/app.elf
	cp target/$(FOLDER)/release/ironfish-dkg.hex bin/app.hex
	cp target/$(FOLDER)/release/ironfish-dkg.apdu bin/app.apdu

.PHONY: read-params-from-json
read-params-from-json:
	ls -la target/$(FOLDER)/release
	$(eval T_ID := $(shell jq -r '.targetId' target/$(FOLDER)/release/app_$(FOLDER).json))
	$(eval DATA_SIZE := $(shell jq -r '.dataSize' target/$(FOLDER)/release/app_$(FOLDER).json))
	$(eval API_LEVEL := $(shell jq -r '.apiLevel' target/$(FOLDER)/release/app_$(FOLDER).json))
	$(eval APPVERSION := $(shell sed -n 's/^APPVERSION = "\(.*\)"/\1/p' .cargo/config.toml))

	$(eval APP_LOAD_PARAMS := $(APP_LOAD_PARAMS)  --appVersion $(APPVERSION) --targetId $(T_ID) --dataSize $(DATA_SIZE) --apiLevel $(API_LEVEL))
	$(eval APP_DELETE_PARAMS := $(APP_DELETE_PARAMS) --targetId $(T_ID))


# This was taken from the Makefile.app_params in the ledger-secure-sdk
ICON_HEX_FILE := $(CURDIR)/icon.hex
ICONHEX_CMD := $(if $(filter $(TARGET_NAME),TARGET_NANOX TARGET_NANOS2), \
        python3 $(CURDIR)/../deps/ledger-secure-sdk/icon3.py --hexbitmaponly $(ICONNAME) > $(ICON_HEX_FILE), \
        python3 $(CURDIR)/../deps/ledger-secure-sdk/lib_nbgl/tools/icon2glyph.py --hexbitmap $(ICON_HEX_FILE) $(ICONNAME))

.PHONY: get-icon-hex
get-icon-hex:
    ICONHEX := $(shell $(ICONHEX_CMD) && cat $(ICON_HEX_FILE) && rm -f $(ICON_HEX_FILE))
    APP_LOAD_PARAMS += --icon $(ICONHEX)