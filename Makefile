.DEFAULT_GOAL := build
plugins_dir := /var/lib/coolercontrol/plugins
executable := macsmc-fans
service_id := macsmc-fans

.PHONY: build clean install

clean:
	@-$(RM) -rf target
	@-$(RM) -rf vendor

target/release/$(executable): build

build:
	@cargo build --locked --release

install: target/release/$(executable)
	@mkdir -p $(DESTDIR)$(plugins_dir)/$(service_id)
	@install -m755 target/release/$(executable) $(DESTDIR)$(plugins_dir)/$(service_id)
	@install -m644 manifest.toml $(DESTDIR)$(plugins_dir)/$(service_id)

run: target/release/$(executable)
	@sudo target/release/$(executable)

uninstall:
	@-sudo $(RM) -rf $(plugins_dir)/$(service_id)
