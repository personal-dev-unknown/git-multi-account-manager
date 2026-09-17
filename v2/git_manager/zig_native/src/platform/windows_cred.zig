// zig_native/src/platform/windows_cred.zig
//
// ── Purpose ───────────────────────────────────────────────────────────────────
// Complete Windows Credential Manager implementation using the Win32
// CredWriteW / CredReadW / CredDeleteW / CredFree API surface from Advapi32.dll.
//
// This file is only compiled when the Zig target is Windows (selected by
// platform/index.zig via comptime). Cross-compilation from Linux/macOS to
// Windows is fully supported — Zig bundles the Windows SDK headers and Zig's
// build system links against Advapi32 automatically when the extern "advapi32"
// declarations are present.
//
// ── Credential storage model ──────────────────────────────────────────────────
// The Windows Credential Store uses a flat namespace of (TargetName, Type)
// pairs. We use CRED_TYPE_GENERIC for all Git Manager credentials. The target
// name format is:
//
//   "git-manager/{label}/{username}"
//
// stored as a UTF-16 null-terminated wide string (the W suffix on all Win32
// functions signals wide-character / UTF-16 variants). This format uniquely
// identifies each credential and is visible in the Windows Credential Manager
// UI: Control Panel → Credential Manager → Windows Credentials.
//
// ── Memory ownership contract ─────────────────────────────────────────────────
// CRITICAL: CredReadW allocates the returned CREDENTIALW with LocalAlloc, which
// is an internal Windows allocator separate from the standard C heap. This buffer
// MUST be freed with CredFree(), never with free() or LocalFree() or any other
// allocator. Using the wrong deallocator causes silent heap corruption. This
// implementation handles this correctly using defer CredFree(cred_ptr) immediately
// after a successful CredReadW call.
//
// ── Calling convention ────────────────────────────────────────────────────────
// Win32 APIs on x86-32 use stdcall (WINAPI). On x86-64 and ARM64, the Microsoft
// calling convention IS the C calling convention — there is only one on 64-bit
// Windows. Zig's .C callconv resolves to the correct platform ABI on all targets,
// making it the right choice for both 32-bit and 64-bit Windows builds.

const std = @import("std");
const kernel32 = std.os.windows.kernel32;

const FsResult = @import("../filesystem/atomic_write.zig").FsResult;

/// Result buffer for secret retrieval.
/// Must match GmSecretResult in gm_native.h and FfiSecretResult in Rust ffi.rs.
pub const SecretResult = extern struct {
    secret:        [4096]u8 = std.mem.zeroes([4096]u8),
    secret_len:    usize    = 0,
    error_message: [512]u8  = std.mem.zeroes([512]u8),
    error_len:     usize    = 0,
};

// ── Win32 type aliases ────────────────────────────────────────────────────────
// We define only the types needed for the Credential Manager API, using
// C-compatible primitive types for correct ABI mapping.

const DWORD = u32;
const BOOL  = c_int;

// Windows Credential Manager constants
const CRED_TYPE_GENERIC:          DWORD = 1;
const CRED_PERSIST_LOCAL_MACHINE: DWORD = 2;
const CRED_FLAGS_NONE:            DWORD = 0;
// Win32 GetLastError code for "element not found"
const ERROR_NOT_FOUND: DWORD = 1168;

/// Win32 FILETIME — 100-nanosecond intervals since January 1, 1601.
/// Required by the CREDENTIALW struct layout. The OS populates this on write;
/// we initialize it to zero and never read it.
const FILETIME = extern struct {
    dwLowDateTime:  u32 = 0,
    dwHighDateTime: u32 = 0,
};

/// Win32 CREDENTIAL_ATTRIBUTEW — optional user-defined key/value attributes.
/// We set AttributeCount = 0 and Attributes = null, but the struct must be
/// declared to satisfy the CREDENTIALW layout requirement.
const CREDENTIAL_ATTRIBUTEW = extern struct {
    Keyword:   [*:0]u16,
    Flags:     DWORD,
    ValueSize: DWORD,
    Value:     [*]u8,
};

/// Win32 CREDENTIALW — the complete credential record in the Windows store.
/// All string fields use UTF-16 wide characters (the W suffix).
/// Field order matches the MSDN specification exactly — any reordering would
/// corrupt reads and writes because CredReadW returns a pointer to this layout.
const CREDENTIALW = extern struct {
    Flags:              DWORD                   = CRED_FLAGS_NONE,
    Type:               DWORD,
    TargetName:         [*:0]u16,
    Comment:            ?[*:0]const u16         = null,
    LastWritten:        FILETIME                = .{},
    CredentialBlobSize: DWORD,
    CredentialBlob:     [*]u8,
    Persist:            DWORD,
    AttributeCount:     DWORD                   = 0,
    Attributes:         ?*CREDENTIAL_ATTRIBUTEW = null,
    TargetAlias:        ?[*:0]u16               = null,
    UserName:           ?[*:0]u16               = null,
};

// ── Win32 API extern declarations ─────────────────────────────────────────────
// "advapi32" tells Zig's linker to resolve these from Advapi32.dll.

extern "advapi32" fn CredWriteW(
    Credential: *const CREDENTIALW,
    Flags:      DWORD,
) callconv(.C) BOOL;

extern "advapi32" fn CredReadW(
    TargetName: [*:0]const u16,
    Type:       DWORD,
    Flags:      DWORD,
    Credential: **CREDENTIALW,
) callconv(.C) BOOL;

extern "advapi32" fn CredDeleteW(
    TargetName: [*:0]const u16,
    Type:       DWORD,
    Flags:      DWORD,
) callconv(.C) BOOL;

/// Frees the CREDENTIALW buffer allocated by CredReadW.
/// MUST be used in place of any standard free — the Windows credential allocator
/// is separate from the C runtime heap.
extern "advapi32" fn CredFree(Buffer: *anyopaque) callconv(.C) void;

// ── Shared internal helpers ───────────────────────────────────────────────────

/// Builds the canonical UTF-16 null-terminated target name:
///   "git-manager/{label}/{username}"
/// The caller must free the returned slice with allocator.free().
fn buildTargetName(
    label:     []const u8,
    username:  []const u8,
    allocator: std.mem.Allocator,
) ![:0]u16 {
    const utf8 = try std.fmt.allocPrint(
        allocator,
        "git-manager/{s}/{s}",
        .{ label, username },
    );
    defer allocator.free(utf8);
    return std.unicode.utf8ToUtf16LeAllocZ(allocator, utf8);
}

/// Writes a Win32 error code diagnostic into a FsResult error buffer.
fn writeFsWin32Error(result: *FsResult, operation: []const u8, code: DWORD) void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    const msg = std.fmt.allocPrint(
        allocator, "{s} failed with Win32 error {d}", .{ operation, code },
    ) catch {
        // On OOM, just write the operation name — avoid freeing non-heap pointer.
        const len = @min(operation.len, result.error_message.len - 1);
        @memcpy(result.error_message[0..len], operation[0..len]);
        result.error_message[len] = 0;
        result.error_len = len;
        return;
    };
    defer allocator.free(msg);
    const len = @min(msg.len, result.error_message.len - 1);
    @memcpy(result.error_message[0..len], msg[0..len]);
    result.error_message[len] = 0;
    result.error_len = len;
}

/// Writes a Win32 error code diagnostic into a SecretResult error buffer.
fn writeSecretWin32Error(result: *SecretResult, operation: []const u8, code: DWORD) void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();
    const msg = std.fmt.allocPrint(
        allocator, "{s} failed with Win32 error {d}", .{ operation, code },
    ) catch {
        // On OOM, just write the operation name — avoid freeing non-heap pointer.
        const len = @min(operation.len, result.error_message.len - 1);
        @memcpy(result.error_message[0..len], operation[0..len]);
        result.error_message[len] = 0;
        result.error_len = len;
        return;
    };
    defer allocator.free(msg);
    const len = @min(msg.len, result.error_message.len - 1);
    @memcpy(result.error_message[0..len], msg[0..len]);
    result.error_message[len] = 0;
    result.error_len = len;
}

fn writeFsError(result: *FsResult, msg: []const u8) void {
    const len = @min(msg.len, result.error_message.len - 1);
    @memcpy(result.error_message[0..len], msg[0..len]);
    result.error_message[len] = 0;
    result.error_len = len;
}

fn writeSecretError(result: *SecretResult, msg: []const u8) void {
    const len = @min(msg.len, result.error_message.len - 1);
    @memcpy(result.error_message[0..len], msg[0..len]);
    result.error_message[len] = 0;
    result.error_len = len;
}

// ── Exported C ABI functions ──────────────────────────────────────────────────

/// Stores a secret in the Windows Credential Store via CredWriteW.
/// CredWriteW performs a natural upsert: if a credential with the same target
/// name already exists it is overwritten atomically. The CredentialBlob field
/// accepts arbitrary bytes, making it suitable for PATs, OAuth tokens, and
/// SSH passphrases alike.
pub export fn gm_platform_store_secret(
    label:      [*:0]const u8,
    username:   [*:0]const u8,
    secret:     [*]const u8,
    secret_len: usize,
    result:     *FsResult,
) bool {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const target_w = buildTargetName(
        std.mem.span(label),
        std.mem.span(username),
        allocator,
    ) catch {
        writeFsError(result, "out of memory building credential target name");
        return false;
    };
    defer allocator.free(target_w);

    // CredWriteW only reads from CredentialBlob; the const cast is safe here.
    const cred = CREDENTIALW{
        .Type               = CRED_TYPE_GENERIC,
        .TargetName         = target_w.ptr,
        .CredentialBlobSize = @intCast(secret_len),
        .CredentialBlob     = @constCast(secret),
        .Persist            = CRED_PERSIST_LOCAL_MACHINE,
    };

    if (CredWriteW(&cred, CRED_FLAGS_NONE) == 0) {
        writeFsWin32Error(result, "CredWriteW", kernel32.GetLastError());
        return false;
    }

    return true;
}

/// Retrieves a secret from the Windows Credential Store via CredReadW.
/// The memory returned by CredReadW is freed with CredFree — this is enforced
/// by a defer immediately after the successful API call so the release cannot
/// be accidentally removed during future edits.
pub export fn gm_platform_retrieve_secret(
    label:    [*:0]const u8,
    username: [*:0]const u8,
    result:   *SecretResult,
) bool {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const target_w = buildTargetName(
        std.mem.span(label),
        std.mem.span(username),
        allocator,
    ) catch {
        writeSecretError(result, "out of memory building credential target name");
        return false;
    };
    defer allocator.free(target_w);

    var cred_ptr: *CREDENTIALW = undefined;

    if (CredReadW(target_w.ptr, CRED_TYPE_GENERIC, CRED_FLAGS_NONE, &cred_ptr) == 0) {
        const code = kernel32.GetLastError();
        if (code == ERROR_NOT_FOUND) {
            writeSecretError(result, "credential not found in Windows Credential Store");
        } else {
            writeSecretWin32Error(result, "CredReadW", code);
        }
        return false;
    }
    // CRITICAL: defer CredFree immediately after successful CredReadW.
    // cred_ptr was allocated by LocalAlloc inside the Win32 subsystem.
    // Freeing it with any other function corrupts the process heap silently.
    defer CredFree(cred_ptr);

    const blob_len = cred_ptr.CredentialBlobSize;
    if (blob_len == 0) {
        writeSecretError(result, "credential exists but CredentialBlob is empty");
        return false;
    }

    const copy_len = @min(@as(usize, blob_len), result.secret.len - 1);
    @memcpy(result.secret[0..copy_len], cred_ptr.CredentialBlob[0..copy_len]);
    result.secret_len = copy_len;

    return true;
}

/// Removes a credential from the Windows Credential Store via CredDeleteW.
/// Returns true if the credential was deleted or if it was not present —
/// deleting a non-existent credential is not an error condition.
pub export fn gm_platform_delete_secret(
    label:    [*:0]const u8,
    username: [*:0]const u8,
    result:   *FsResult,
) bool {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const target_w = buildTargetName(
        std.mem.span(label),
        std.mem.span(username),
        allocator,
    ) catch {
        writeFsError(result, "out of memory building credential target name");
        return false;
    };
    defer allocator.free(target_w);

    if (CredDeleteW(target_w.ptr, CRED_TYPE_GENERIC, CRED_FLAGS_NONE) == 0) {
        const code = kernel32.GetLastError();
        // ERROR_NOT_FOUND means the credential did not exist — treat as success.
        if (code == ERROR_NOT_FOUND) return true;
        writeFsWin32Error(result, "CredDeleteW", code);
        return false;
    }

    return true;
}