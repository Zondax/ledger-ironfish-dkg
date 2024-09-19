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
COIN=IRON
endif

include $(CURDIR)/Makefile.version

$(info COIN  = [$(COIN)])

ifeq ($(COIN),IRON)
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
copy_icon:
	cp $(DEFAULT_ICON) glyphs/icon_app.gif

all: copy_icon

$(info TARGET_NAME  = [$(TARGET_NAME)])
$(info ICONNAME  = [$(ICONNAME)])

ifndef ICONNAME
$(error ICONNAME is not set)
endif

APPVERSION=$(APPVERSION_M).$(APPVERSION_N).$(APPVERSION_P)
APPVERSION_STR=v$(APPVERSION)

include $(CURDIR)/../deps/ledger-zxlib/makefiles/Makefile.side_loading

.PHONY: buildS
buildS:
	APPVERSION_STR=$(APPVERSION_STR) PRODUCTION_BUILD=$(PRODUCTION_BUILD) cargo ledger build nanos
	FOLDER=nanos make post-build

.PHONY: buildX
buildX:
	APPVERSION_STR=$(APPVERSION_STR) PRODUCTION_BUILD=$(PRODUCTION_BUILD) cargo ledger build nanox
	FOLDER=nanox make post-build

.PHONY: buildS2
buildS2:
	APPVERSION_STR=$(APPVERSION_STR) PRODUCTION_BUILD=$(PRODUCTION_BUILD) cargo ledger build nanosplus
	FOLDER=nanosplus make post-build

.PHONY: buildST
buildST:
	APPVERSION_STR=$(APPVERSION_STR) PRODUCTION_BUILD=$(PRODUCTION_BUILD) cargo ledger build stax
	FOLDER=stax make post-build


.PHONY: buildFL
buildFL:
	APPVERSION_STR=$(APPVERSION_STR) PRODUCTION_BUILD=$(PRODUCTION_BUILD) cargo ledger build flex
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


.PHONY: post-build
post-build: move-files-to-output read-params-from-json create-install-script

.PHONY: move-files-to-output
move-files-to-output:
	cp target/$(FOLDER)/release/ledger-ironfish-dkg bin/app.elf
	cp target/$(FOLDER)/release/ledger-ironfish-dkg.hex bin/app.hex
	cp target/$(FOLDER)/release/ledger-ironfish-dkg.apdu bin/app.apdu

.PHONY: read-params-from-json
read-params-from-json:
	ls -la target/$(FOLDER)/release
	$(eval T_ID := $(shell jq -r '.targetId' app/target/$(FOLDER)/release/app_$(FOLDER).json))
	$(eval DATA_SIZE := $(shell jq -r '.dataSize' app/target/$(FOLDER)/release/app_$(FOLDER).json))
	$(eval API_LEVEL := $(shell jq -r '.apiLevel' app/target/$(FOLDER)/release/app_$(FOLDER).json))

	$(eval APP_LOAD_PARAMS := $(APP_LOAD_PARAMS) --targetId $(T_ID) --dataSize $(DATA_SIZE) --apiLevel $(API_LEVEL))
	$(eval APP_DELETE_PARAMS := $(APP_DELETE_PARAMS) --targetId $(T_ID))
