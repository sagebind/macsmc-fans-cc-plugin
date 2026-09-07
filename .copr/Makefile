SPEC_FILE := packaging/fedora/rpm.spec
PKG_NAME := $(shell awk 'NR==1 {print $$2}' $(SPEC_FILE))
VERSION := $(shell awk -F'"' '/^version = / {print $$2; exit;}' Cargo.toml)
SOURCE_NAME := $(PKG_NAME)-$(VERSION)
SOURCE_ARCHIVE := $(SOURCE_NAME).tar.gz
TAR_ARGS := --exclude=.git --exclude=target

srpm:
	mkdir -p $(outdir)
	tar czvf $(outdir)/$(SOURCE_ARCHIVE) $(TAR_ARGS) --transform='s,^.,$(SOURCE_NAME),' .
	sed 's/__VERSION__/$(VERSION)/' $(SPEC_FILE) > $(outdir)/rpm.spec
	rpmbuild -bs \
		--define "_sourcedir $(outdir)" \
		--define "_srcrpmdir $(outdir)" \
		$(outdir)/rpm.spec
