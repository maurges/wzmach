CARGO = cargo

PREFIX = /usr/bin
BUILT_TARGET = target/release/wzmach

LOCAL_PREFIX = $(HOME)/.local/bin
LOCAL_USER = $(USER)


target/release/wzmach:
	"$(CARGO)" build --release

# You probably have cargo installed locally, and you will run install as root,
# so it won't find cargo
install:
	install "$(BUILT_TARGET)" "$(PREFIX)/wzmach" --group=input --owner=root --mode=2755

install-local: target/release/wzmach
	sudo install $< "$(LOCAL_PREFIX)/wzmach" --group=input "--owner=$(LOCAL_USER)" --mode=2754

uninstall:
	rm "$(PREFIX)/wzmach"

.PHONY: target/release/wzmach install install-local uninstall
