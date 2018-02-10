TAG := srwalker101/rustfitsioversion
VERSION := 3420
FULLTAG := $(TAG):$(VERSION)

build:
	docker build -t $(FULLTAG) .

shell:
	docker run --rm -it \
		--mount type=bind,source="$(shell pwd)/../..",target=/fitsiotest \
		--mount type=bind,source="$(shell pwd)/../.cargo/registry",target=/root/.cargo/registry \
		$(FULLTAG) \
		/bin/sh

run:
	docker run --rm -it \
		--mount type=bind,source="$(shell pwd)/../..",target=/fitsiotest \
		--mount type=bind,source="$(shell pwd)/../.cargo/registry",target=/root/.cargo/registry \
		$(FULLTAG)
