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

### Consumer dependency (Percussion / VJ)

One natives JAR carries **both Windows x64 and Linux x64** shared libraries.
The SDK extracts the matching one at runtime.

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
  <!-- no classifier: main JAR includes windows-x86_64 + linux-x86_64 -->
</dependency>
```

Optional platform-only classifiers (smaller) if you want to pin one OS:

- `windows-x86_64`
- `linux-x86_64`

You can still set `jna.library.path` to a system install instead of depending
on `mkd-gcm-natives`.

### Build the reactor

From the **bindings/** directory (parent POM):

```bash
# Packages Win64 + Linux64 natives (Linux via WSL when building on Windows)
mvn -q clean package
mvn -q test
```

Host-only (faster; Windows DLL only on a Windows machine):

```bash
mvn -pl java-natives package -Dnative.hostOnly=true
```

| Platform | Library file | How it is built (Windows host) |
|----------|--------------|--------------------------------|
| `windows-x86_64` | `mkd_gcm_ffi.dll` | host `cargo build -p mkd-gcm-ffi --release` |
| `linux-x86_64` | `libmkd_gcm_ffi.so` | **WSL** cargo with `CARGO_TARGET_DIR=target/linux-x86_64` |

Resource path inside the natives JAR:

`dev/monkeyking/gcm/native/<platform>/<libfile>`

Header: `include/mkd_gcm.h` (repo root).

## Publish to Maven Central

Namespace / groupId: **`dev.monkeyking`**.

### Local credentials

`~/.m2/settings.xml` must define a server with id **`mkd-central`** (Sonatype
Central Publisher Portal user token — not your login password). Use a separate
server id (e.g. **`hashseal-central`**) for other product lines so tokens stay
isolated:

```xml
<settings>
  <servers>
    <server>
      <id>mkd-central</id>
      <username><!-- MonkeyKing / mkd Portal token username --></username>
      <password><!-- token password --></password>
    </server>
    <!-- other products, e.g. HashSeal:
    <server>
      <id>hashseal-central</id>
      <username><!-- … --></username>
      <password><!-- … --></password>
    </server>
    -->
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
