use gm_shared::errors::GitManagerError;

/// Defines the set of capabilities a session may exercise.
///
/// In single-user mode (the current deployment model), every session holds
/// `PermissionSet::all()` — full access to all commands. For multi-user or
/// team editions, individual permissions can be revoked to implement
/// role-based access control without changing any calling code.
#[derive(Debug, Clone)]
pub struct PermissionSet {
    /// If true, all permission checks pass without consulting the individual flags.
    /// This is the v1 default — the local user is the data owner.
    pub unrestricted: bool,
    pub account:      PermissionGroup,
    pub ssh:          PermissionGroup,
    pub repository:   PermissionGroup,
    pub git_ops:      PermissionGroup,
    pub config:       PermissionGroup,
    pub admin:        PermissionGroup,
}

#[derive(Debug, Clone)]
pub struct PermissionGroup {
    pub list:   bool,
    pub read:   bool,
    pub create: bool,
    pub update: bool,
    pub delete: bool,
}

impl PermissionSet {
    /// Returns a permission set that grants all operations.
    /// This is the default for single-user mode.
    pub fn all() -> Self {
        Self {
            unrestricted: true,
            account:      PermissionGroup::all(),
            ssh:          PermissionGroup::all(),
            repository:   PermissionGroup::all(),
            git_ops:      PermissionGroup::all(),
            config:       PermissionGroup::all(),
            admin:        PermissionGroup::all(),
        }
    }

    /// Returns a permission set that denies all operations.
    /// Used as a starting point for locked-down configurations.
    pub fn none() -> Self {
        Self {
            unrestricted: false,
            account:      PermissionGroup::none(),
            ssh:          PermissionGroup::none(),
            repository:   PermissionGroup::none(),
            git_ops:      PermissionGroup::none(),
            config:       PermissionGroup::none(),
            admin:        PermissionGroup::none(),
        }
    }
}

impl PermissionGroup {
    pub fn all() -> Self {
        Self { list: true, read: true, create: true, update: true, delete: true }
    }

    pub fn none() -> Self {
        Self { list: false, read: false, create: false, update: false, delete: false }
    }

    fn allows(&self, operation: &str) -> bool {
        match operation {
            "list"   => self.list,
            "read"   | "get"    => self.read,
            "create" | "add" | "generate" => self.create,
            "update" | "set" | "store" => self.update,
            "delete" | "remove" => self.delete,
            _ => false,
        }
    }
}

/// Maps a fully-qualified command type name to a (resource, operation) pair.
///
/// The command_type string comes from `std::any::type_name::<C>()` which returns
/// the Rust path (e.g. `gm_ports::inbound::commands::add_account::AddAccountCommand`).
/// The mapping uses a suffix match on the last component of the path.
fn classify_command(command_type: &str) -> (&'static str, &'static str) {
    let cmd = command_type.rsplit(':').next().unwrap_or(command_type);
    match cmd {
        // Account commands
        s if s.contains("AddAccount")       => ("account",  "create"),
        s if s.contains("RemoveAccount")    => ("account",  "delete"),
        s if s.contains("ListAccount")      => ("account",  "list"),
        s if s.contains("GetAccount")       => ("account",  "read"),
        s if s.contains("SetDefaultAccount") => ("account", "update"),
        s if s.contains("StoreAccountToken") => ("account", "update"),
        // SSH commands
        s if s.contains("GenerateSshKey")   => ("ssh",      "create"),
        s if s.contains("TestSshConnection") => ("ssh",     "read"),
        s if s.contains("AddKeyToAgent")    => ("ssh",      "update"),
        s if s.contains("ListSshKey")       => ("ssh",      "list"),
        // Repository commands
        s if s.contains("CloneRepository")  => ("repository", "create"),
        // Git operation commands
        s if s.contains("PullRepository")   => ("git_ops",  "update"),
        s if s.contains("PushRepository")   => ("git_ops",  "update"),
        s if s.contains("ListOperation")    => ("git_ops",  "list"),
        // Config commands
        s if s.contains("Config")           => ("config",   "update"),
        s if s.contains("ListConfig")       => ("config",   "list"),
        // Fallback
        _                                   => ("admin",    "read"),
    }
}

/// The access-control gatekeeper.
///
/// In single-user mode (v1 default), the checker uses `PermissionSet::all()`
/// which permits every command. For multi-user or team editions, replace the
/// default set with a restricted one via `with_permissions()` and the command
/// bus will enforce the new boundaries without any other code changes.
#[derive(Debug)]
pub struct PermissionChecker {
    permissions: PermissionSet,
}

impl PermissionChecker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Replaces the active permission set. Call this before processing any
    /// user requests to switch from permissive to restricted mode.
    pub fn with_permissions(mut self, perms: PermissionSet) -> Self {
        self.permissions = perms;
        self
    }

    /// Returns Ok(()) if the current session is allowed to execute the named
    /// command type. Returns Err with a descriptive message if denied.
    ///
    /// The `command_type` parameter is the string produced by
    /// `std::any::type_name::<C>()` — e.g.
    /// `"gm_ports::inbound::commands::add_account::AddAccountCommand"`.
    pub fn check_command(&self, command_type: &str) -> Result<(), GitManagerError> {
        if self.permissions.unrestricted {
            return Ok(());
        }

        let (resource, operation) = classify_command(command_type);

        let group = match resource {
            "account"    => &self.permissions.account,
            "ssh"        => &self.permissions.ssh,
            "repository" => &self.permissions.repository,
            "git_ops"    => &self.permissions.git_ops,
            "config"     => &self.permissions.config,
            "admin"      => &self.permissions.admin,
            _            => &self.permissions.admin,
        };

        if group.allows(operation) {
            Ok(())
        } else {
            Err(GitManagerError::Other(format!(
                "Permission denied: {} operation on {} resource",
                operation, resource,
            )))
        }
    }
}

impl Default for PermissionChecker {
    fn default() -> Self {
        Self {
            permissions: PermissionSet::all(),
        }
    }
}
