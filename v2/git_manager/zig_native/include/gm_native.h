/*
 * gm_native.h — C API for the Git Manager native library (libgm_native.a)
 *
 * ── How this file is generated ────────────────────────────────────────────────
 * Zig automatically emits a C header from exported functions and extern structs
 * when the library is built with `zig build`. The emitted header is installed to
 * zig-out/include/gm_native.h. This checked-in version documents the INTENDED
 * API contract; the emitted version is the ground truth for type sizes.
 *
 * ── How to use this file from Rust ────────────────────────────────────────────
 * Do NOT include this file from Rust directly. Instead, write corresponding
 * #[repr(C)] structs in gm_adapters/src/*/ffi.rs that match the layout here.
 * Rust's extern "C" { } blocks declare the function signatures.
 *
 * If you change a struct in a Zig source file:
 *   1. Update the struct definition in the Zig file
 *   2. Update the struct definition here
 *   3. Update the matching #[repr(C)] struct in the Rust ffi.rs file
 *   4. Run `zig build` to regenerate zig-out/include/gm_native.h
 *   5. Diff the regenerated header against this file — they must match
 *
 * ── Memory ownership ──────────────────────────────────────────────────────────
 * ALL result structs are caller-allocated. The caller creates the struct (zeroed),
 * passes a pointer, and reads the result fields after the function returns.
 * The Zig library NEVER allocates memory that the caller must free.
 * All data lives in fixed-size byte arrays inside the result structs.
 *
 * ── Thread safety ─────────────────────────────────────────────────────────────
 * All functions are safe to call from DIFFERENT threads simultaneously, provided
 * they operate on different resources (different key paths, different repo paths).
 * Functions operating on the SAME resource concurrently (e.g., two calls writing
 * to the same ~/.ssh/config) are NOT safe — the Rust domain layer serializes these.
 *
 * ── Calling convention ────────────────────────────────────────────────────────
 * All functions use the C calling convention. On x86-64 Linux and macOS, this
 * is the System V AMD64 ABI. On AArch64, this is the ARM64 ABI.
 * Every function returns a bool: true = success, false = failure.
 * On failure, the result struct's error_message field contains the diagnostic.
 */

#pragma once

#include <stddef.h>   /* size_t                    */
#include <stdint.h>   /* uint8_t, uint32_t, etc.   */
#include <stdbool.h>  /* bool, true, false          */

#ifdef __cplusplus
extern "C" {
#endif

/* ════════════════════════════════════════════════════════════════════════════
 * SSH RESULT TYPES
 * ════════════════════════════════════════════════════════════════════════════ */

/**
 * GmKeygenResult — output of gm_ssh_generate_key()
 *
 * Buffer sizing:
 *   public_key  [4096] — Ed25519 keys are ~100 chars; RSA-4096 keys are ~740 chars.
 *                        4096 bytes handles any future key types with room to spare.
 *   fingerprint [256]  — SHA256 fingerprints are "SHA256:" + 43 base64 chars = 50 chars.
 *   error_message [512]— enough for any ssh-keygen error including full file paths.
 *
 * Rust mirror: #[repr(C)] struct FfiKeygenResult in gm_adapters/src/ssh/ffi.rs
 * Total size: 4096 + 8 + 256 + 8 + 512 + 8 = 4888 bytes (on 64-bit systems)
 */
typedef struct {
    uint8_t public_key[4096];     /* complete public key string, null-terminated  */
    size_t  public_key_len;       /* number of bytes, excluding null terminator    */
    uint8_t fingerprint[256];     /* "SHA256:base64..." null-terminated            */
    size_t  fingerprint_len;      /* number of bytes, excluding null terminator    */
    uint8_t error_message[512];   /* populated only when the function returns false */
    size_t  error_len;            /* number of bytes in error_message              */
} GmKeygenResult;

/**
 * GmAgentResult — output of gm_ssh_add_to_agent() and gm_ssh_remove_from_agent()
 *
 * Rust mirror: #[repr(C)] struct FfiAgentResult in gm_adapters/src/ssh/ffi.rs
 * Total size: 512 + 8 = 520 bytes
 */
typedef struct {
    uint8_t error_message[512];
    size_t  error_len;
} GmAgentResult;

/**
 * GmConnResult — output of gm_ssh_test_connection()
 *
 * The `success` field is true when the server responded with a known
 * authentication success pattern (e.g., "Hi username!" from GitHub).
 * The `username` field contains the username extracted from the server response.
 * The `response` field contains the full server response for display to the user.
 *
 * Rust mirror: #[repr(C)] struct FfiConnResult in gm_adapters/src/ssh/ffi.rs
 * Total size: 1 + padding + 256 + 8 + 1024 + 8 + 512 + 8 bytes
 */
typedef struct {
    bool    success;              /* true if authentication was confirmed          */
    uint8_t username[256];        /* authenticated username, null-terminated       */
    size_t  username_len;
    uint8_t response[1024];       /* full server response, null-terminated         */
    size_t  response_len;
    uint8_t error_message[512];
    size_t  error_len;
} GmConnResult;

/* ════════════════════════════════════════════════════════════════════════════
 * FILESYSTEM RESULT TYPES
 * ════════════════════════════════════════════════════════════════════════════ */

/**
 * GmFsResult — shared result type for filesystem operations that produce
 * no data on success (atomic_write, set_permissions, write_config_entry, etc.)
 *
 * Rust mirror: #[repr(C)] struct FfiFsResult in gm_adapters/src/filesystem/ffi.rs
 * Total size: 512 + 8 = 520 bytes
 */
typedef struct {
    uint8_t error_message[512];
    size_t  error_len;
} GmFsResult;

/**
 * GmPathResult — output of gm_fs_expand_path()
 *
 * Rust mirror: #[repr(C)] struct FfiPathResult in gm_adapters/src/filesystem/ffi.rs
 * Total size: 2048 + 8 + 512 + 8 = 2576 bytes
 */
typedef struct {
    uint8_t path[2048];           /* expanded absolute path, null-terminated      */
    size_t  path_len;
    uint8_t error_message[512];
    size_t  error_len;
} GmPathResult;

/* ════════════════════════════════════════════════════════════════════════════
 * GIT EXECUTION RESULT TYPES
 * ════════════════════════════════════════════════════════════════════════════ */

/**
 * GmGitResult — output of git operation functions (clone, pull, push, commit, stage)
 *
 * Buffer sizing:
 *   commit_sha  [64]    — SHA-1 is 40 hex chars; SHA-256 is 64 hex chars.
 *   stdout_data [65536] — 64 KiB captures git clone progress and push summaries.
 *                         Operations producing more output are truncated.
 *   stderr_data [4096]  — 4 KiB captures any error message from git or SSH.
 *
 * Rust mirror: #[repr(C)] struct FfiGitResult in gm_adapters/src/git/ffi.rs
 */
typedef struct {
    bool    success;
    uint8_t commit_sha[64];       /* HEAD commit SHA after the operation           */
    size_t  sha_len;
    uint8_t stdout_data[65536];   /* git subprocess stdout, possibly truncated     */
    size_t  stdout_len;
    uint8_t stderr_data[4096];    /* git subprocess stderr / error diagnostics     */
    size_t  stderr_len;
    int32_t exit_code;            /* raw git process exit code (-1 if killed)      */
    int32_t child_pid;            /* PID of last git subprocess; -1 if none        */
} GmGitResult;

/**
 * GmGitStatus — output of gm_git_status()
 *
 * Each "files" field contains newline-separated file paths from
 * `git status --porcelain=v1`. The corresponding "count" field
 * is the number of entries (== number of newlines in the buffer).
 *
 * Buffer sizing [32768 each]: 32 KiB supports repositories with thousands of
 * simultaneously modified files. Repositories with more than ~32,000 modified
 * files have the file list truncated but the counts remain accurate.
 *
 * Rust mirror: #[repr(C)] struct FfiGitStatus in gm_adapters/src/git/ffi.rs
 */
typedef struct {
    uint8_t  staged_files[32768];     /* newline-separated staged file paths       */
    size_t   staged_len;
    uint32_t staged_count;

    uint8_t  unstaged_files[32768];   /* newline-separated unstaged file paths     */
    size_t   unstaged_len;
    uint32_t unstaged_count;

    uint8_t  untracked_files[32768];  /* newline-separated untracked file paths    */
    size_t   untracked_len;
    uint32_t untracked_count;

    bool     is_clean;                /* true when all three counts are zero       */
    uint8_t  error_message[512];
    size_t   error_len;
} GmGitStatus;

/* ════════════════════════════════════════════════════════════════════════════
 * PLATFORM CREDENTIAL RESULT TYPES
 * ════════════════════════════════════════════════════════════════════════════ */

/**
 * GmSecretResult — output of gm_platform_retrieve_secret()
 *
 * The `secret` field contains the raw secret bytes (not null-terminated,
 * because secrets may contain arbitrary bytes including null bytes).
 * The `secret_len` field indicates how many bytes are valid.
 *
 * Security: on the Rust side, this struct must be zeroed immediately after
 * reading the secret bytes into a secure memory region. The struct should
 * not be stored or logged.
 *
 * Rust mirror: #[repr(C)] struct FfiSecretResult in gm_adapters/src/platform/ffi.rs
 */
typedef struct {
    uint8_t secret[4096];         /* raw secret bytes (may contain null bytes)    */
    size_t  secret_len;           /* valid byte count in secret[]                 */
    uint8_t error_message[512];
    size_t  error_len;
} GmSecretResult;

/* ════════════════════════════════════════════════════════════════════════════
 * SSH FUNCTIONS
 * ════════════════════════════════════════════════════════════════════════════ */

/**
 * gm_ssh_generate_key — generates an SSH key pair using the system ssh-keygen.
 *
 * @param key_type   null-terminated: "ed25519", "rsa", "rsa-2048", or "ecdsa"
 * @param email      null-terminated: embedded in the key comment field
 * @param out_path   null-terminated: absolute path for the private key file
 * @param passphrase null-terminated: key passphrase, or "" for no passphrase
 * @param result     caller-allocated, must be valid, zeroed before call
 * @return           true on success; result.public_key and result.fingerprint populated
 *                   false on failure; result.error_message populated
 */
bool gm_ssh_generate_key(
    const char*    key_type,
    const char*    email,
    const char*    out_path,
    const char*    passphrase,
    GmKeygenResult* result
);

/**
 * gm_ssh_agent_running — checks whether an SSH agent is accessible.
 *
 * Detection: SSH_AUTH_SOCK environment variable is set and non-empty.
 * Does NOT attempt a socket connection. A true return means an agent
 * MIGHT be running; gm_ssh_add_to_agent will confirm it definitively.
 *
 * @return true if SSH_AUTH_SOCK is set and non-empty
 */
bool gm_ssh_agent_running(void);

/**
 * gm_ssh_add_to_agent — loads an SSH private key into the running agent.
 *
 * @param key_path   null-terminated: absolute path to the private key file
 * @param passphrase null-terminated: key passphrase, or "" for passwordless keys
 * @param result     caller-allocated, must be valid
 * @return           true if the key was successfully loaded
 */
bool gm_ssh_add_to_agent(
    const char*  key_path,
    const char*  passphrase,
    GmAgentResult* result
);

/**
 * gm_ssh_remove_from_agent — removes an SSH key from the running agent.
 *
 * Returns true if the key was removed OR if it was not in the agent.
 * Returns false only on agent communication failure.
 *
 * @param key_path   null-terminated: absolute path to the private key file
 * @param result     caller-allocated, must be valid
 */
bool gm_ssh_remove_from_agent(
    const char*  key_path,
    GmAgentResult* result
);

/**
 * gm_ssh_test_connection — tests SSH authentication to a Git hosting platform.
 *
 * Runs: ssh -T -i key_path -o IdentitiesOnly=yes -o BatchMode=yes host
 * Parses the server response for known success patterns from GitHub, GitLab,
 * Bitbucket, and Azure DevOps.
 *
 * @param host       null-terminated: SSH host alias or hostname (e.g. "github.com-work")
 * @param key_path   null-terminated: absolute path to the private key
 * @param timeout_ms connection timeout in milliseconds (minimum 1000)
 * @param result     caller-allocated, must be valid
 * @return           true if server confirmed authentication; result.username populated
 */
bool gm_ssh_test_connection(
    const char*  host,
    const char*  key_path,
    uint32_t     timeout_ms,
    GmConnResult* result
);

/**
 * gm_ssh_write_config_entry — appends a Host block to ~/.ssh/config.
 *
 * Idempotent: if the host_alias already exists in the config, returns true
 * without modifying the file.
 * Atomic: uses temp-file + rename to prevent config corruption on crash.
 *
 * @param host_alias     null-terminated: SSH host alias, e.g. "github.com-work"
 * @param hostname       null-terminated: actual hostname, e.g. "github.com"
 * @param identity_file  null-terminated: absolute path to private key
 * @param port           SSH port (usually 22)
 * @param result         caller-allocated, must be valid
 */
bool gm_ssh_write_config_entry(
    const char* host_alias,
    const char* hostname,
    const char* identity_file,
    uint16_t    port,
    GmFsResult* result
);

/**
 * gm_ssh_remove_config_entry — removes a Host block from ~/.ssh/config.
 *
 * Returns true if removed, or if the alias was not present.
 *
 * @param host_alias  null-terminated: SSH host alias to remove
 * @param result      caller-allocated, must be valid
 */
bool gm_ssh_remove_config_entry(
    const char* host_alias,
    GmFsResult* result
);

/* ════════════════════════════════════════════════════════════════════════════
 * GIT EXECUTION FUNCTIONS
 * ════════════════════════════════════════════════════════════════════════════ */

/**
 * gm_git_kill_process — kills a running git subprocess by PID.
 *
 * Pass `pid=0` to kill the currently tracked child (uses an internal atomic
 * global set by the most recent `runChild()` call). This is the recommended
 * usage from the Ctrl+C handler in Rust.
 *
 * @param pid  target PID, or 0 to kill the currently tracked child
 * @return     true if a kill signal was sent, false if nothing to kill
 */
bool gm_git_kill_process(int32_t pid);

/**
 * gm_git_clone — clones a remote repository into a local directory.
 *
 * Uses GIT_SSH_COMMAND with the specified key to isolate SSH identity.
 * Equivalent to: GIT_SSH_COMMAND="ssh -i key_path -o IdentitiesOnly=yes ..."
 *                git clone --progress url dest
 *
 * @param url       null-terminated: SSH or HTTPS remote URL
 * @param dest      null-terminated: absolute local destination path (must not exist)
 * @param key_path  null-terminated: absolute path to the SSH private key
 * @param result    caller-allocated GmGitResult, must be valid
 */
bool gm_git_clone(
    const char* url,
    const char* dest,
    const char* key_path,
    GmGitResult* result
);

/**
 * gm_git_pull — pulls the latest commits from the remote.
 *
 * @param repo_path  null-terminated: absolute path to the local git repository
 * @param key_path   null-terminated: absolute path to the SSH private key
 * @param rebase     when true, uses --rebase instead of --merge
 * @param result     caller-allocated GmGitResult, must be valid
 */
bool gm_git_pull(
    const char* repo_path,
    const char* key_path,
    bool        rebase,
    GmGitResult* result
);

/**
 * gm_git_push — pushes local commits to the remote.
 *
 * @param repo_path  null-terminated: absolute path to the local git repository
 * @param remote     null-terminated: remote name, usually "origin"
 * @param branch     null-terminated: branch name, usually "main"
 * @param key_path   null-terminated: absolute path to the SSH private key
 * @param force      when true, uses --force-with-lease (safer than --force)
 * @param result     caller-allocated GmGitResult, must be valid
 */
bool gm_git_push(
    const char* repo_path,
    const char* remote,
    const char* branch,
    const char* key_path,
    bool        force,
    GmGitResult* result
);

/**
 * gm_git_commit — creates a commit with currently staged changes.
 *
 * Overrides GIT_AUTHOR_NAME and GIT_AUTHOR_EMAIL to ensure the commit is
 * attributed to the correct account regardless of global git config.
 * Caller MUST stage changes first with gm_git_stage_all or equivalent.
 *
 * @param repo_path    null-terminated: absolute path to the local repository
 * @param message      null-terminated: commit message
 * @param author_name  null-terminated: git author name for the commit object
 * @param author_email null-terminated: git author email for the commit object
 * @param result       caller-allocated GmGitResult, must be valid
 */
bool gm_git_commit(
    const char* repo_path,
    const char* message,
    const char* author_name,
    const char* author_email,
    GmGitResult* result
);

/**
 * gm_git_status — retrieves working tree status using git status --porcelain=v1.
 *
 * @param repo_path  null-terminated: absolute path to the local git repository
 * @param result     caller-allocated GmGitStatus, must be valid
 */
bool gm_git_status(
    const char* repo_path,
    GmGitStatus* result
);

/**
 * gm_git_stage_all — stages all changes (equivalent to git add -A).
 *
 * Stages new files, modified files, and deleted files. Does not stage
 * files matched by .gitignore.
 *
 * @param repo_path  null-terminated: absolute path to the local git repository
 * @param result     caller-allocated GmGitResult, must be valid
 */
bool gm_git_stage_all(
    const char* repo_path,
    GmGitResult* result
);

/* ════════════════════════════════════════════════════════════════════════════
 * FILESYSTEM FUNCTIONS
 * ════════════════════════════════════════════════════════════════════════════ */

/**
 * gm_fs_atomic_write — writes data to a file using temp-file + rename.
 *
 * The write is atomic at the filesystem level: the target file is either
 * fully written with new content or unchanged. There is no intermediate state.
 * The written file has permissions 0600.
 *
 * @param path    null-terminated: absolute path to the target file
 * @param data    pointer to the bytes to write (must point to at least `len` bytes)
 * @param len     number of bytes to write
 * @param result  caller-allocated GmFsResult, must be valid
 */
bool gm_fs_atomic_write(
    const char*    path,
    const uint8_t* data,
    size_t         len,
    GmFsResult*    result
);

/**
 * gm_fs_set_permissions — sets the permission mode of a file or directory.
 *
 * @param path    null-terminated: absolute path to the file or directory
 * @param mode    POSIX permission mode (e.g. 0600 for SSH private keys)
 * @param result  caller-allocated GmFsResult, must be valid
 */
bool gm_fs_set_permissions(
    const char* path,
    uint32_t    mode,
    GmFsResult* result
);

/**
 * gm_fs_expand_path — expands ~ to the HOME directory value.
 *
 * Does NOT access the filesystem. Does NOT resolve symlinks.
 * Use this to build absolute paths before the file exists.
 *
 * @param input   null-terminated: path that may start with ~ or ~/
 * @param result  caller-allocated GmPathResult, must be valid
 */
bool gm_fs_expand_path(
    const char*  input,
    GmPathResult* result
);

/**
 * gm_fs_ensure_dir — creates a directory and all parent directories.
 *
 * Idempotent: returns true if the directory already exists.
 * Sets the final directory's permissions to `mode` after creation.
 *
 * @param path    null-terminated: absolute path to the directory
 * @param mode    POSIX permission mode for the created directory
 * @param result  caller-allocated GmFsResult, must be valid
 */
bool gm_fs_ensure_dir(
    const char* path,
    uint32_t    mode,
    GmFsResult* result
);

/* ════════════════════════════════════════════════════════════════════════════
 * PLATFORM CREDENTIAL STORAGE FUNCTIONS
 *
 * Platform selection at build time (comptime in Zig):
 *   Linux  → GNOME libsecret D-Bus API (requires libsecret-1-dev)
 *   macOS  → macOS Security.framework Keychain API
 *   Other  → Encrypted file fallback (~/.git-manager/secrets.enc, AES-256-GCM)
 * ════════════════════════════════════════════════════════════════════════════ */

/**
 * gm_platform_store_secret — stores a secret in the platform keychain.
 *
 * The secret is identified by the combination of (label, username).
 * If a secret with the same (label, username) already exists, it is overwritten.
 *
 * @param label       null-terminated: human-readable label, e.g. "git-manager/work-pat"
 * @param username    null-terminated: account identifier, e.g. "shakamoses"
 * @param secret      pointer to the secret bytes (may contain null bytes)
 * @param secret_len  number of bytes in secret
 * @param result      caller-allocated GmFsResult, must be valid
 */
bool gm_platform_store_secret(
    const char*    label,
    const char*    username,
    const uint8_t* secret,
    size_t         secret_len,
    GmFsResult*    result
);

/**
 * gm_platform_retrieve_secret — retrieves a secret from the platform keychain.
 *
 * @param label    null-terminated: the label used when storing the secret
 * @param username null-terminated: the username used when storing the secret
 * @param result   caller-allocated GmSecretResult, must be valid and zeroed.
 *                 SECURITY: zero the result struct immediately after reading
 *                 the secret bytes — do not store the struct.
 * @return         true if the secret was found and written to result.secret
 *                 false if not found or on keychain access error
 */
bool gm_platform_retrieve_secret(
    const char*   label,
    const char*   username,
    GmSecretResult* result
);

/**
 * gm_platform_delete_secret — removes a secret from the platform keychain.
 *
 * Returns true if the secret was deleted or if it did not exist.
 *
 * @param label    null-terminated: the label used when storing the secret
 * @param username null-terminated: the username used when storing the secret
 * @param result   caller-allocated GmFsResult, must be valid
 */
bool gm_platform_delete_secret(
    const char* label,
    const char* username,
    GmFsResult* result
);

#ifdef __cplusplus
} /* extern "C" */
#endif