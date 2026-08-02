mkd-gcm-natives
===============

Platform shared libraries for the GCM Java SDK (libmkd_gcm_ffi / mkd_gcm_ffi).

Layout inside this JAR:

  dev/monkeyking/gcm/native/<platform>/<library-file>

Platforms (classifier names match os-maven-plugin):

  windows-x86_64   → mkd_gcm_ffi.dll
  linux-x86_64     → libmkd_gcm_ffi.so
  linux-aarch_64   → libmkd_gcm_ffi.so
  osx-x86_64       → libmkd_gcm_ffi.dylib
  osx-aarch_64     → libmkd_gcm_ffi.dylib

The Java SDK (mkd-gcm-sdk) extracts the matching library at runtime when
jna.library.path is not already set. You can also depend on the classified
artifact explicitly:

  <dependency>
    <groupId>dev.monkeyking</groupId>
    <artifactId>mkd-gcm-natives</artifactId>
    <version>VERSION</version>
    <classifier>windows-x86_64</classifier>
  </dependency>

Build (from repository root):

  cargo build -p mkd-gcm-ffi --release
  cd bindings && mvn -pl java-natives package

Skip cargo (use an existing release build):

  mvn -pl java-natives package -Dcargo.skip=true
