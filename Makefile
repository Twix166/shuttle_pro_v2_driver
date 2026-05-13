obj-m += hid-shuttlepro.o

KDIR ?= /lib/modules/$(shell uname -r)/build
PWD := $(CURDIR)
DKMS_NAME := hid-shuttlepro
DKMS_VERSION := 0.1.0

all:
	$(MAKE) -C $(KDIR) M=$(PWD) modules

clean:
	$(MAKE) -C $(KDIR) M=$(PWD) clean

install:
	$(MAKE) -C $(KDIR) M=$(PWD) INSTALL_MOD_DIR=extra modules_install
	depmod -a

uninstall:
	rm -f /lib/modules/$(shell uname -r)/extra/hid-shuttlepro.ko*
	depmod -a

dkms-add:
	dkms add .

dkms-build:
	dkms build $(DKMS_NAME)/$(DKMS_VERSION)

dkms-install:
	dkms install $(DKMS_NAME)/$(DKMS_VERSION)

dkms-remove:
	dkms remove $(DKMS_NAME)/$(DKMS_VERSION) --all

.PHONY: all clean install uninstall dkms-add dkms-build dkms-install dkms-remove
