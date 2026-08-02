# mkd-gcm Java SDK (thin)

Thin JNA wrapper over **`libmkd_gcm_ffi`** (Rust `mkd-gcm` → GCM network).

**License:** MIT — Copyright (c) 2026 MonkeyKing.dev  
**Coordinates:** `dev.monkeyking:mkd-gcm-sdk:0.2.0`

JVM host applications hold the GCM token **server-side**, call this SDK, and
never expose the PAT to the browser.

## Maven coordinates

| Artifact | Role |
|----------|------|
| `dev.monkeyking:mkd-gcm-sdk` | Pure Java / JNA API |
| `dev.monkeyking:mkd-gcm-natives` | Shared library (`mkd_gcm_ffi`) for the build host; also classifier JARs |
| `dev.monkeyking:mkd-gcm-sdk-jackson2` | Optional Jackson 2.x helpers |
| `dev.monkeyking:mkd-gcm-sdk-jackson3` | Optional Jackson 3.x helpers |

### Consumer dependency

```xml
<dependency>
  <groupId>dev.monkeyking</groupId>
  <artifactId>mkd-gcm-sdk</artifactId>
  <version>0.2.0</version>
</dependency>
<dependency>
  <groupId>dev.monkeyking</groupId>
  <artifactId>mkd-gcm-natives</artifactId>
  <version>0.2.0</version>
  <!-- optional but recommended: pin the platform classifier -->
  <!-- <classifier>windows-x86_64</classifier> -->
  <!-- <classifier>linux-x86_64</classifier> -->
  <!-- <classifier>osx-aarch_64</classifier> -->
</dependency>
```

The SDK extracts the matching native from the classpath at startup when
`jna.library.path` is not set. You can still ship a system library and set
`jna.library.path` instead of depending on `mkd-gcm-natives`.

### Build the reactor

From the **bindings/** directory (parent POM):

```bash
# builds cargo mkd-gcm-ffi --release, packages natives + SDK + Jackson modules
mvn -q clean package
mvn -q test
```

Skip cargo when the release FFI is already built:

```bash
mvn -pl java-natives package -Dcargo.skip=true
```

| OS | Cargo artifact | Classifier (os-maven) |
|----|----------------|------------------------|
| Linux x86_64 | `libmkd_gcm_ffi.so` | `linux-x86_64` |
| Linux aarch64 | `libmkd_gcm_ffi.so` | `linux-aarch_64` |
| macOS Intel | `libmkd_gcm_ffi.dylib` | `osx-x86_64` |
| macOS Apple Silicon | `libmkd_gcm_ffi.dylib` | `osx-aarch_64` |
| Windows x64 | `mkd_gcm_ffi.dll` | `windows-x86_64` |

Resource path inside the natives JAR:

`dev/monkeyking/gcm/native/<classifier>/<libfile>`

Header: `include/mkd_gcm.h` (repo root).

## Publish to Maven Central

Namespace / groupId: **`dev.monkeyking`**.

### Local credentials

`~/.m2/settings.xml` must define a server with id **`central`** (Sonatype
Central Publisher Portal user token — not your login password):

```xml
<settings>
  <servers>
    <server>
      <id>central</id>
      <username><!-- token username --></username>
      <password><!-- token password --></password>
    </server>
  </servers>
</settings>
```

### GPG signing

Central requires GPG signatures. Create a key if needed, publish the public key
to a keyserver, then:

```bash
cd bindings
mvn clean deploy -Pcentral
# auto-publish after validation:
mvn clean deploy -Pcentral -Dcentral.autoPublish=true
```

Without a GPG key the `central` profile will fail at the sign step. Set
`-Dgpg.skip=true` only for local packaging experiments (will not be accepted by
Central).

### What gets deployed

- Parent POM `mkd-gcm-sdk-parent`
- `mkd-gcm-natives` (+ platform classifier JAR for the build host)
- `mkd-gcm-sdk`
- `mkd-gcm-sdk-jackson2`
- `mkd-gcm-sdk-jackson3`

Multi-OS natives: build/deploy on each platform (or CI matrix) so classifier
JARs for linux/osx/windows are all present on Central.

### Optional: GitHub Packages

```bash
mvn clean deploy -Pgithub-packages
```

(server id `github` in `settings.xml`)

## Host REST sketch (generic Java app)

```java
// Browser posts CorrectionSubmission JSON to your REST endpoint.
// Token comes from server config / secrets — never from the client.

try (GcmClient client = GcmClient.connect("gcm.monkeyking.dev", 1119)) {
    client.setBearerToken(serverConfig.getProjectSubmitToken());
    client.setDefaultFrom("language-bot@example.com");

    CorrectionSubmission sub = /* deserialize request body */;
    String mid = client.postCorrection(
        "mkd.gcm.orgs.org.example.language.projects.crowdsource-demo.inbound",
        sub);
}
```

Browser path: language plugin → `postUrl` → this REST handler → SDK → GCM.

## Layout

```text
bindings/
  pom.xml                 parent (Central / GPG profiles)
  java/                   mkd-gcm-sdk
  java-natives/           mkd-gcm-natives (FFI shared libs)
  java-jackson2/
  java-jackson3/
```
