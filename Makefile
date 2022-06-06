CARGO = cargo
TARGET = $(HOME)/.local/bin

.PHONY: target/release/wzmach install

target/release/wzmach:
	$(CARGO) build --release
	strip $@

install: target/release/wzmach
	cp $< "$(TARGET)/wzmach"
	sudo chmod 754 "$(TARGET)/wzmach"
	sudo chown $(USER):input "$(TARGET)/wzmach"
	sudo chmod g+s "$(TARGET)/wzmach"
