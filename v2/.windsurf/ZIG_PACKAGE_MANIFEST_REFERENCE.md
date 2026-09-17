# Zig Package Manifest Reference (`*.zon` files)

> **Why this document exists:** `.zon` files (Zig Object Notation) are Zig's
> package manifests. Many editors treat them as hidden or configuration files —
> similar to `.env` — and file explorers may suppress them from default views.
> This document preserves the exact content of both `.zon` files so you always
> have a readable copy.
>
> **If you ever lose or corrupt a `.zon` file**, copy the content from the
> corresponding block below, paste it into the correct path, and save it.
> Do not add or remove any commas, dots, or quotes — the format is strict.

---

## File 1 — `git_manager/build.zig.zon` 

**File type:** Zig workspace package manifest  
**Role:** Declares the root Zig package and declares `zig_native` as a local
path dependency. The root `build.zig` calls `b.dependency("zig_native", ...)` 
which Zig resolves by reading this file's `.dependencies.zig_native.path` field.  
**Location in project:** `git_manager/build.zig.zon` (workspace root, same level as `Cargo.toml`)

```zon
.{
    // Root workspace Zig package manifest.
    // This is the Zig equivalent of the root Cargo.toml's [workspace] section.
    // It declares the top-level project name and declares zig_native as a
    // local path dependency so that `b.dependency("zig_native", ...)` in
    // build.zig resolves correctly.
    .name    = "git_manager",
    .version = "0.1.0",

    .dependencies = .{
        // zig_native is a LOCAL path dependency — no content hash needed.
        // Path dependencies are resolved relative to this file's directory.
        // When build.zig calls b.dependency("zig_native", .{...}), Zig reads
        // zig_native/build.zig.zon to confirm it is a valid Zig package.
        .zig_native = .{
            .path = "zig_native",
        },
    },

    // The paths that are included when this package is published or used as
    // a dependency. We include zig_native/ so that consumers of this package
    // also get the native layer sources.
    .paths = .{
        "build.zig",
        "build.zig.zon",
        "zig_native/",
    },
}
```

---

## File 2 — `git_manager/zig_native/build.zig.zon` 

**File type:** Zig library package manifest  
**Role:** Declares the `gm_native` static library package. When the root
`build.zig` resolves the `zig_native` dependency, Zig reads this file to confirm
the sub-project is a valid package and to check that its declared `.paths` are
present on disk.  
**Location in project:** `git_manager/zig_native/build.zig.zon` 

```zon
.{
    // Package metadata for the Zig native layer.
    // The .name and .version are used when this package is referenced
    // as a dependency from the root build.zig.
    .name    = "gm_native",
    .version = "0.1.0",

    // Zig does not yet have a centralized package registry like crates.io.
    // All external dependencies must be declared here with their exact
    // content hash so builds are reproducible. Currently gm_native has
    // no external Zig dependencies — everything is done via the standard
    // library and system library bindings (libsecret, Security.framework).
    .dependencies = .{},

    // Paths included when this package is used as a dependency.
    // Only the src/ directory and build files are needed; include/ is
    // generated output and should not be distributed as source.
    .paths = .{
        "build.zig",
        "build.zig.zon",
        "src/",
    },
}
```

---

## Quick recovery checklist

If either file is missing when you run `zig build` from the workspace root,
you will see an error like:

```
error: unable to find 'build.zig.zon'
```

or:

```
error: dependency 'zig_native' not found
```

Recovery steps are:
1. Create the missing file at the path shown in the File header above.
2. Paste the content from the corresponding code block (the `zon` fenced block).
3. Save the file.
4. Run `zig build` again from `git_manager/`.

No hash needs to be computed for local path dependencies — only for remote
URL dependencies fetched from the internet.
