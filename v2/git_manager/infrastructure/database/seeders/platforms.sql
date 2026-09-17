-- Platform seed data — also in the migration file.
-- Kept here for documentation and manual seeding if needed.
INSERT IGNORE INTO platforms (uuid, name, display_name, api_base_url, ssh_host) VALUES
  ('00000000-0001-0000-0000-000000000001', 'github',        'GitHub',          'https://api.github.com',        'github.com'),
  ('00000000-0002-0000-0000-000000000001', 'gitlab',        'GitLab',          'https://gitlab.com/api/v4',     'gitlab.com'),
  ('00000000-0003-0000-0000-000000000001', 'bitbucket',     'Bitbucket',       'https://api.bitbucket.org/2.0', 'bitbucket.org'),
  ('00000000-0004-0000-0000-000000000001', 'azure_devops',  'Azure DevOps',    'https://dev.azure.com',         'ssh.dev.azure.com'),
  ('00000000-0005-0000-0000-000000000001', 'sourceforge',   'SourceForge',     'https://sourceforge.net/rest',  'git.code.sf.net'),
  ('00000000-0006-0000-0000-000000000001', 'self_hosted',   'Self-Hosted',     NULL,                            NULL),
  ('00000000-0007-0000-0000-000000000001', 'cloud_storage', 'Cloud Storage',   NULL,                            NULL),
  ('00000000-0008-0000-0000-000000000001', 'local_path',    'Local Path',      NULL,                            NULL),
  ('00000000-0009-0000-0000-000000000001', 'custom',        'Custom',          NULL,                            NULL);
