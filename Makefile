IMAGE_TAG ?= loom:latest
IMAGE_ARCHIVE ?= dist/loom.tar.gz
IMAGE_PLATFORM ?= linux/amd64
LOOM_SOURCES := $(shell find . \
	-path ./target -prune -o \
	-path ./loom-bv/target -prune -o \
	-path ./dist -prune -o \
	-path ./.git -prune -o \
	-type f -print)
DOCKER_INPUTS := Dockerfile $(LOOM_SOURCES)

.PHONY: docker docker-archive

docker: $(IMAGE_ARCHIVE)

docker-archive: docker

$(IMAGE_ARCHIVE): $(DOCKER_INPUTS)
	mkdir -p $(dir $(IMAGE_ARCHIVE))
	docker build --platform $(IMAGE_PLATFORM) -f Dockerfile -t $(IMAGE_TAG) .
	docker save $(IMAGE_TAG) | gzip > $(IMAGE_ARCHIVE)
	@echo "wrote $(IMAGE_ARCHIVE)"
