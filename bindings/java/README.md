# mkd-gcm Java SDK (thin)

Thin JNA wrapper over **`libmkd_gcm_ffi`** (Rust `mkd-gcm` → GCM network).

**License:** MIT — Copyright (c) 2026 MonkeyKing.dev

JVM host applications hold the GCM token **server-side**, call this SDK, and
never expose the PAT to the browser.

## Build native library

From the **mkd-gcm-client** repo root:

```bash
cargo build -p mkd-gcm-ffi --release
```

| OS | Artifact |
|----|----------|
| Linux | `target/release/libmkd_gcm_ffi.so` |
| macOS | `target/release/libmkd_gcm_ffi.dylib` |
| Windows | `target/release/mkd_gcm_ffi.dll` |

Header: `include/mkd_gcm.h` (repo root).

### Installing on the target machine

1. Copy the matching shared library for the host OS/CPU into a directory the
   process can load (e.g. `/usr/local/lib`, application `lib/`, or Windows
   install dir).
2. Ensure the JVM can find it:
   - `-Djna.library.path=/path/to/dir` (recommended for apps), or
   - system library path (`LD_LIBRARY_PATH`, `PATH` on Windows, etc.)
3. Ship the JAR (`mkd-gcm-sdk`) on the classpath.
4. Optional: package the `.so`/`.dll` inside your application installer or
   container image next to the service.

There is no separate “GCM installer” yet — host ops copy the native lib + JAR.

## Maven

```bash
cd bindings/java
mvn -q test
mvn -q package
```

### GitHub Packages

Artifacts publish to GitHub Packages (`maven.pkg.github.com/monkeyking-hq/mkd-gcm-client`)
until Maven Central is configured. Authenticate with a PAT that has
`read:packages` (and `write:packages` for deploy).

```xml
<repositories>
  <repository>
    <id>github</id>
    <url>https://maven.pkg.github.com/monkeyking-hq/mkd-gcm-client</url>
  </repository>
</repositories>

<dependency>
  <groupId>dev.monkeyking</groupId>
  <artifactId>mkd-gcm-sdk</artifactId>
  <version>0.2.0</version>
</dependency>
```

Maintainers: `mvn -DskipTests deploy` (server id `github` in `settings.xml`) or
the **Publish Java to GitHub Packages** GitHub Actions workflow.

Optional JSON mappers (if you already use Jackson):

- `dev.monkeyking:mkd-gcm-sdk-jackson2` — Jackson 2.x
- `dev.monkeyking:mkd-gcm-sdk-jackson3` — Jackson 3.x

The core module has **no** Jackson dependency; `CorrectionSubmission.toJson()`
works stand-alone. POJO field names are camelCase for the browser language
plugin JSON contract.

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
bindings/java/
  pom.xml
  src/main/java/dev/monkeyking/gcm/
    GcmClient.java
    GcmException.java
    CorrectionSubmission.java
    NativeLib.java
```
