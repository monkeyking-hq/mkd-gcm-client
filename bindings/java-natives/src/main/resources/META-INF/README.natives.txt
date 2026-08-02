mkd-gcm-natives
===============

Shared libraries for the GCM Java SDK (mkd_gcm_ffi), embedded as classpath
resources and extracted at runtime by mkd-gcm-sdk.

Supported platforms (current product requirement)
-------------------------------------------------
  windows-x86_64   → mkd_gcm_ffi.dll
  linux-x86_64     → libmkd_gcm_ffi.so

Layout inside this JAR
----------------------
  dev/monkeyking/gcm/native/<platform>/<library-file>

Consumer dependency (both platforms in the main JAR)
----------------------------------------------------
  <dependency>
    <groupId>dev.monkeyking</groupId>
    <artifactId>mkd-gcm-natives</artifactId>
    <version>VERSION</version>
  </dependency>

Optional platform-only classifiers (smaller downloads if you pin one OS):
  …:mkd-gcm-natives:VERSION:windows-x86_64
  …:mkd-gcm-natives:VERSION:linux-x86_64

Build / stage
-------------
  From repo root (Windows, packages both Win+Linux via WSL for the .so):

    cd bindings
    mvn -pl java-natives package

  Host only (faster local loop):

    mvn -pl java-natives package -Dnative.hostOnly=true

  Scripts:
    scripts/stage-natives.ps1
    scripts/stage-natives.sh
