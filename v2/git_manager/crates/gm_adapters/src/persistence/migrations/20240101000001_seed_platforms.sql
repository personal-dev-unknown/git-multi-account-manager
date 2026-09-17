-- 20240101000001_seed_platforms.sql
--
-- Seeds the initial platform registry rows.
-- These are the Git hosting platforms supported out of the box.
-- Each platform entry corresponds to a provider plugin crate.
-- INSERT IGNORE ensures idempotency if this migration is re-run accidentally.

SET FOREIGN_KEY_CHECKS = 0;

INSERT IGNORE INTO platforms (uuid, name, display_name, api_base_url, ssh_host, supports_oauth, supports_pat, supports_ssh) VALUES
    ('00000000-0001-0000-0000-000000000001', 'github',        'GitHub',             'https://api.github.com',          'github.com',         1, 1, 1),
    ('00000000-0002-0000-0000-000000000001', 'gitlab',        'GitLab',             'https://gitlab.com/api/v4',       'gitlab.com',         1, 1, 1),
    ('00000000-0003-0000-0000-000000000001', 'bitbucket',     'Bitbucket',          'https://api.bitbucket.org/2.0',   'bitbucket.org',      1, 1, 1),
    ('00000000-0004-0000-0000-000000000001', 'azure_devops',  'Azure DevOps',       'https://dev.azure.com',           'ssh.dev.azure.com',  0, 1, 1),
    ('00000000-0005-0000-0000-000000000001', 'sourceforge',   'SourceForge',        'https://sourceforge.net/rest',    'git.code.sf.net',    0, 0, 1),
    ('00000000-0006-0000-0000-000000000001', 'self_hosted',   'Self-Hosted',        NULL,                              NULL,                 0, 1, 1),
    ('00000000-0007-0000-0000-000000000001', 'cloud_storage', 'Cloud Storage',      NULL,                              NULL,                 0, 0, 0),
    ('00000000-0008-0000-0000-000000000001', 'local_path',    'Local Path',         NULL,                              NULL,                 0, 0, 0),
    ('00000000-0009-0000-0000-000000000001', 'custom',        'Custom',             NULL,                              NULL,                 0, 1, 1);

-- Default application configuration values
INSERT IGNORE INTO app_configurations (config_key, config_value, data_type, description, namespace) VALUES
    ('ssh.default_key_type',     '"ed25519"',   'string',  'Default SSH key type for new key generation',    'app'),
    ('ssh.connect_timeout_ms',   '10000',       'integer', 'SSH connection test timeout in milliseconds',    'app'),
    ('ssh.auto_add_to_agent',    'true',        'boolean', 'Automatically add new SSH keys to the agent',   'app'),
    ('git.max_concurrent_ops',   '4',           'integer', 'Maximum concurrent git operations',              'app'),
    ('ui.show_banner',           'true',        'boolean', 'Show the startup banner',                       'app'),
    ('ui.log_level',             '"info"',      'string',  'Application log level (trace/debug/info/warn/error)', 'app');

SET FOREIGN_KEY_CHECKS = 1;