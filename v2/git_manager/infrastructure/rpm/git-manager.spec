Name:           git-manager
Version:        0.1.0
Release:        1%{?dist}
Summary:        Multi-account Git manager
License:        MIT
URL:            https://github.com/your-org/git-manager

Requires:       libsecret, openssh, git

%description
Manage multiple GitHub, GitLab, Bitbucket, and Azure DevOps accounts from
a single CLI, web, or desktop interface with automatic SSH key management.

%install
mkdir -p %{buildroot}/usr/bin
install -m 0755 target/release/git-manager     %{buildroot}/usr/bin/git-manager
install -m 0755 target/release/git-manager-web %{buildroot}/usr/bin/git-manager-web

%files
/usr/bin/git-manager
/usr/bin/git-manager-web
