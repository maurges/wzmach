CARGO = cargo

PREFIX = /usr/bin
CONFIG_PREFIX = /etc
BUILT_TARGET = target/release/wzmach
CONFIG_TARGET = config.ron

LOCAL_PREFIX = $(HOME)/.local/bin
LOCAL_USER = $(USER)


target/release/wzmach:
	"$(CARGO)" build --release

# You probably have cargo installed locally, and you will run install as root,
# so it won't find cargo. So we don't put the binary as a dependency
install:
	mkdir -p "$(PREFIX)"
	install "$(BUILT_TARGET)" "$(PREFIX)/wzmach" --group=input --owner=root --mode=2755
	mkdir -p "$(CONFIG_PREFIX)/wzmach"
	install "$(CONFIG_TARGET)" "$(CONFIG_PREFIX)/wzmach/config.ron" --group=root --owner=root --mode=644

install-local: $(BUILT_TARGET)
	mkdir -p "$(LOCAL_PREFIX)"
	sudo install $< "$(LOCAL_PREFIX)/wzmach" --group=input "--owner=$(LOCAL_USER)" --mode=2754

autostart: wzmach.desktop
	install wzmach.desktop "$(LOCAL_USER)/.config/autostart/wzmach.desktop"

uninstall:
	rm "$(PREFIX)/wzmach"
	rm "$(CONFIG_PREFIX)/wzmach/config.ron"
	rmdir "$(CONFIG_PREFIX)/wzmach"

.PHONY: target/release/wzmach install install-local uninstall
