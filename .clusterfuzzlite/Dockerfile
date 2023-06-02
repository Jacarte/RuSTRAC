FROM gcr.io/oss-fuzz-base/base-builder-rust

RUN apt-get update && apt-get install -y

COPY . $SRC/fvm
WORKDIR $SRC/fvm
RUN ls -R $SRC/fvm

COPY ./.clusterfuzzlite/build.sh $SRC/
