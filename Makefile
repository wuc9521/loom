IMAGE_TAG ?= delis:bench
IMAGE_ARCHIVE ?= dist/delis.tar.gz
IMAGE_PLATFORM ?= linux/amd64
PARENT_DIR := $(abspath ..)
DELIS_SOURCES := $(shell find . \
	-path ./target -prune -o \
	-path ./dist -prune -o \
	-path ./.git -prune -o \
	-type f -print)
SYGUS_SOURCES := $(shell cd $(PARENT_DIR)/sygus-parser-rs && find . \
	-path ./target -prune -o \
	-path ./.git -prune -o \
	-type f -print | sed 's|^\./|$(PARENT_DIR)/sygus-parser-rs/|')
DRYAD_SOURCES := $(shell cd $(PARENT_DIR)/dryadsynth-bv && find . \
	-path ./target -prune -o \
	-path ./.git -prune -o \
	-type f -print | sed 's|^\./|$(PARENT_DIR)/dryadsynth-bv/|')
DOCKER_INPUTS := Dockerfile $(DELIS_SOURCES) $(SYGUS_SOURCES) $(DRYAD_SOURCES)

.PHONY: docker docker-archive

docker: $(IMAGE_ARCHIVE)

docker-archive: docker

$(IMAGE_ARCHIVE): $(DOCKER_INPUTS)
	mkdir -p $(dir $(IMAGE_ARCHIVE))
	docker build --platform $(IMAGE_PLATFORM) -f Dockerfile -t $(IMAGE_TAG) $(PARENT_DIR)
	docker save $(IMAGE_TAG) | gzip > $(IMAGE_ARCHIVE)
	@echo "wrote $(IMAGE_ARCHIVE)"
