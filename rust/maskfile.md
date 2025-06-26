# Tasks for the project

## build-tabula-rs

> Builds a working version of tabula-java for use with tabula-rs.

```sh
mkdir -p deps
cd deps
if [[ ! -d tabula-rs ]]; then
  git clone git@github.com:sp1ritCS/tabula-rs.git
fi
if [[ ! -d tabula-java ]]; then
  git clone git@github.com:tabulapdf/tabula-java.git
fi
cd tabula-java
patch -p1 < ../tabula-rs/0001-add-ffi-constructor-to-CommandLineApp.patch
rm tabula-*-SNAPSHOT-jar-with-dependencies.jar
mvn compile assembly:single
cd ../..
mkdir -p target/debug target/release
cp deps/tabula-java/target/tabula-*-SNAPSHOT-jar-with-dependencies.jar target/debug/tabula.jar
cp deps/tabula-java/target/tabula-*-SNAPSHOT-jar-with-dependencies.jar target/release/tabula.jar
```
