# Clone System - Integration Architecture

## System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                    User Interfaces                              │
├──────────────────────┬──────────────────────┬──────────────────┤
│   CLI Interactive    │   Web Browser        │   Desktop GUI    │
│   (interactive.py)   │   (Flask + REST)     │   (PyQt6)        │
└──────────────────────┴──────────────────────┴──────────────────┘
         │                      │                      │
         └──────────────────────┼──────────────────────┘
                                │
         ┌──────────────────────▼──────────────────────┐
         │         API Layer (clone/api/)              │
         ├──────────────────────────────────────────────┤
         │  CloneAPI  │  RepositoryAPI  │  PlatformAPI │
         └──────────────────────┬──────────────────────┘
                                │
         ┌──────────────────────▼──────────────────────┐
         │      Core Clone System (clone/)             │
         ├──────────────────────────────────────────────┤
         │                                              │
         │  ┌─────────────────────────────────────┐    │
         │  │  CloneWorkflow (Orchestrator)       │    │
         │  │  - fetch_personal_repositories()   │    │
         │  │  - analyze_external_repository()   │    │
         │  │  - clone_repository()              │    │
         │  │  - fork_repository()               │    │
         │  └─────────────────────────────────────┘    │
         │           │              │                   │
         │  ┌────────▼──────┐  ┌────▼─────────┐        │
         │  │ Platforms     │  │ Auth Methods │        │
         │  ├───────────────┤  ├──────────────┤        │
         │  │ • GitHub      │  │ • SSH        │        │
         │  │ • GitLab      │  │ • PAT        │        │
         │  │ • Bitbucket   │  │ • Password   │        │
         │  │ • Custom      │  │ • Anonymous  │        │
         │  └───────────────┘  └──────────────┘        │
         │                                              │
         │  ┌─────────────────────────────────────┐    │
         │  │  Utilities                          │    │
         │  │  - URLParser (parsers.py)          │    │
         │  │  - RepositoryCache (cache.py)      │    │
         │  │  - Custom Errors (errors.py)       │    │
         │  └─────────────────────────────────────┘    │
         │                                              │
         └──────────────────────────────────────────────┘
                                │
         ┌──────────────────────▼──────────────────────┐
         │      System Managers                         │
         ├──────────────────────────────────────────────┤
         │  • AccountManager                            │
         │  • ConfigManager                             │
         │  • SSHManager                                │
         │  • GitOperations                             │
         └──────────────────────────────────────────────┘
                                │
         ┌──────────────────────▼──────────────────────┐
         │      External Systems                        │
         ├──────────────────────────────────────────────┤
         │  • Git (git clone, git config)              │
         │  • SSH (ssh-keygen, ssh-add)                │
         │  • Platform APIs (GitHub, GitLab, etc.)     │
         │  • File System                              │
         └──────────────────────────────────────────────┘
```

## Data Flow

### Clone External Repository Flow

```
User Input (URL, Account, Auth Method)
         │
         ▼
┌─────────────────────────────────────┐
│ URLParser.parse()                   │
│ - Parse URL                         │
│ - Detect platform                   │
│ - Normalize to SSH/HTTPS            │
└─────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────┐
│ CloneWorkflow.clone_repository()    │
│ - Get account                       │
│ - Prepare destination               │
│ - Select auth method                │
└─────────────────────────────────────┘
         │
         ▼
    ┌────┴────┬────────────┬──────────┐
    │          │            │          │
    ▼          ▼            ▼          ▼
┌──────┐  ┌──────┐  ┌──────────┐  ┌──────────┐
│ SSH  │  │ PAT  │  │ Password │  │Anonymous │
│Auth  │  │Auth  │  │ Auth     │  │ Auth     │
└──────┘  └──────┘  └──────────┘  └──────────┘
    │          │            │          │
    └────┬─────┴────────────┴──────────┘
         │
         ▼
┌─────────────────────────────────────┐
│ Execute: git clone <url> <dest>     │
└─────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────┐
│ Post-Clone Setup                    │
│ - Set git user config               │
│ - Configure SSH                     │
│ - Add upstream remote (if fork)     │
└─────────────────────────────────────┘
         │
         ▼
    Success/Error Result
```

### Personal Repository Clone Flow

```
User Selection (Platform, Account, Repo)
         │
         ▼
┌─────────────────────────────────────┐
│ RepositoryCache.get()               │
│ - Check if cached                   │
│ - Return if valid                   │
└─────────────────────────────────────┘
         │
    ┌────┴─────────────────┐
    │ (Cached)             │ (Not Cached)
    │                      │
    ▼                      ▼
Return Cached    ┌─────────────────────────────────────┐
Repositories     │ Platform.fetch_repositories()       │
                 │ - Call API                          │
                 │ - Paginate results                  │
                 │ - Normalize data                    │
                 └─────────────────────────────────────┘
                          │
                          ▼
                 ┌─────────────────────────────────────┐
                 │ RepositoryCache.set()               │
                 │ - Store in cache                    │
                 │ - Set TTL                          │
                 └─────────────────────────────────────┘
                          │
    ┌─────────────────────┘
    │
    ▼
Display Repositories
    │
    ▼
User Selects Repository
    │
    ▼
Clone Selected Repository (same as external flow)
```

## Interface Integration

### CLI Integration

```
interactive.py
    │
    ├─ clone_repo()
    │   │
    │   ├─ _clone_external_repository()
    │   │   └─ Uses CloneWorkflow
    │   │
    │   └─ _clone_personal_repository()
    │       └─ Uses CloneWorkflow
    │
    └─ Rich UI for menus and tables
```

### Web Integration

```
Flask App (web/app.py)
    │
    └─ clone_routes.py (Blueprint)
        │
        ├─ /api/clone/external (POST)
        │   └─ CloneAPI.clone_external_repository()
        │
        ├─ /api/clone/personal (POST)
        │   └─ CloneAPI.clone_personal_repository()
        │
        ├─ /api/clone/repositories (GET)
        │   └─ RepositoryAPI.list_personal_repositories()
        │
        ├─ /api/clone/repositories/search (GET)
        │   └─ RepositoryAPI.search_repositories()
        │
        ├─ /api/clone/repositories/filter (GET)
        │   └─ RepositoryAPI.filter_repositories()
        │
        ├─ /api/clone/platforms (GET)
        │   └─ PlatformAPI.get_supported_platforms()
        │
        ├─ /api/clone/platforms/<id>/test (POST)
        │   └─ PlatformAPI.test_platform_connection()
        │
        └─ ... (more endpoints)
```

### Desktop Integration

```
PyQt6 App (desktop/app.py)
    │
    └─ CloneWidget (desktop/widgets/clone_widget.py)
        │
        ├─ External Repository Tab
        │   ├─ CloneAPI.clone_external_repository()
        │   └─ CloneWorkerThread (async execution)
        │
        ├─ Personal Repository Tab
        │   ├─ RepositoryAPI.list_personal_repositories()
        │   ├─ CloneAPI.clone_personal_repository()
        │   └─ CloneWorkerThread (async execution)
        │
        └─ Platforms Tab
            ├─ PlatformAPI.get_supported_platforms()
            ├─ PlatformAPI.get_platform_accounts()
            └─ PlatformAPI.test_platform_connection()
```

## Module Dependencies

```
┌─────────────────────────────────────────────────────────┐
│ User Interfaces (CLI, Web, Desktop)                     │
└──────────────────────┬──────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────┐
│ API Layer (clone/api/)                                  │
│ ├─ CloneAPI                                             │
│ ├─ RepositoryAPI                                        │
│ └─ PlatformAPI                                          │
└──────────────────────┬──────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────┐
│ Core Clone System (clone/)                              │
│ ├─ CloneWorkflow (orchestrator)                         │
│ ├─ Platforms (GitHub, GitLab, Bitbucket, Custom)       │
│ ├─ Auth Methods (SSH, PAT, Password, Anonymous)        │
│ ├─ URLParser                                            │
│ ├─ RepositoryCache                                      │
│ └─ Custom Errors                                        │
└──────────────────────┬──────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────┐
│ System Managers                                         │
│ ├─ AccountManager                                       │
│ ├─ ConfigManager                                        │
│ ├─ SSHManager                                           │
│ └─ GitOperations                                        │
└──────────────────────┬──────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────┐
│ External Systems                                        │
│ ├─ Git                                                  │
│ ├─ SSH                                                  │
│ ├─ Platform APIs                                        │
│ └─ File System                                          │
└─────────────────────────────────────────────────────────┘
```

## Request Flow Example

### Web API: Clone External Repository

```
1. HTTP Request
   POST /api/clone/external
   {
     "repo_url": "github.com/user/repo",
     "account_name": "my-account",
     "auth_method": "ssh"
   }

2. Flask Route Handler
   clone_routes.clone_external()

3. CloneAPI
   clone_api.clone_external_repository()

4. CloneWorkflow
   workflow.analyze_external_repository()
   workflow.clone_repository()

5. Authentication
   SSHAuth.clone()

6. Git Execution
   subprocess.run(['git', 'clone', ...])

7. Post-Clone Setup
   workflow._setup_post_clone()

8. Response
   {
     "success": true,
     "destination": "/path/to/repo",
     "message": "Clone successful"
   }
```

### Desktop GUI: Clone Personal Repository

```
1. User Action
   - Select Platform: GitHub
   - Select Account: my-account
   - Click "Refresh Repositories"

2. CloneWidget
   repo_api.list_personal_repositories()

3. RepositoryAPI
   - Check cache
   - If not cached: Platform.fetch_repositories()

4. GitHubPlatform
   - Call GitHub API
   - Paginate results
   - Normalize data

5. Display
   - Update table with repositories

6. User Selects Repository
   - Click "Clone Selected Repository"

7. CloneWorkerThread (async)
   clone_api.clone_personal_repository()

8. Progress Signals
   - Update progress bar
   - Display output

9. Completion
   - Show success/error message
```

## Error Handling Flow

```
Clone Operation
    │
    ├─ URLParser.parse() fails
    │   └─ InvalidURLError
    │
    ├─ Account not found
    │   └─ ValueError
    │
    ├─ SSH key not found
    │   └─ SSHError
    │
    ├─ API call fails
    │   └─ APIError
    │
    ├─ Git clone fails
    │   ├─ AuthenticationError
    │   ├─ PermissionError
    │   ├─ RepositoryNotFoundError
    │   ├─ NetworkError
    │   └─ DiskSpaceError
    │
    └─ Post-clone setup fails
        └─ Warning (non-fatal)

All errors caught and returned to user with:
- Error message
- Possible causes
- Suggested solutions
```

## Caching Strategy

```
RepositoryCache
    │
    ├─ Key: {account_id}_{platform}_repos.json
    ├─ Location: ~/.config/git-manager/cache/
    ├─ TTL: 300 seconds (5 minutes)
    │
    └─ Operations:
        ├─ get() - Retrieve cached repos
        ├─ set() - Store repos with timestamp
        ├─ clear() - Clear specific or all cache
        └─ is_expired() - Check if cache expired
```

## Security Architecture

```
┌─────────────────────────────────────┐
│ User Input                          │
└────────────────┬────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────┐
│ Input Validation                    │
│ - URL format check                  │
│ - Account existence check           │
│ - Path validation                   │
└────────────────┬────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────┐
│ Authentication                      │
│ - SSH key verification              │
│ - PAT token validation              │
│ - Connection testing                │
└────────────────┬────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────┐
│ Authorization                       │
│ - Account permissions               │
│ - Repository access check           │
│ - Platform API validation           │
└────────────────┬────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────┐
│ Execution                           │
│ - Secure subprocess execution       │
│ - Environment variable isolation    │
│ - File permission enforcement       │
└────────────────┬────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────┐
│ Post-Execution                      │
│ - Token removal from git config     │
│ - SSH key permission verification   │
│ - Sensitive data cleanup            │
└─────────────────────────────────────┘
```

## Performance Optimization

```
┌─────────────────────────────────────┐
│ Repository Caching                  │
│ - 5-minute TTL                      │
│ - Per-account, per-platform         │
│ - Automatic expiration              │
└─────────────────────────────────────┘

┌─────────────────────────────────────┐
│ Pagination                          │
│ - 100 repos per page                │
│ - Efficient API calls               │
│ - Reduced memory usage              │
└─────────────────────────────────────┘

┌─────────────────────────────────────┐
│ Async Operations (Desktop)          │
│ - Clone in separate thread          │
│ - UI remains responsive             │
│ - Progress updates via signals      │
└─────────────────────────────────────┘

┌─────────────────────────────────────┐
│ Connection Pooling (Web)            │
│ - Reuse HTTP connections            │
│ - Reduce overhead                   │
│ - Faster API calls                  │
└─────────────────────────────────────┘
```

## Summary

The clone system architecture provides:

✅ **Modularity** - Clear separation of concerns
✅ **Scalability** - Easy to extend and maintain
✅ **Consistency** - Same core logic across all interfaces
✅ **Performance** - Optimized with caching and async
✅ **Security** - Best practices throughout
✅ **Reliability** - Comprehensive error handling
✅ **Usability** - Intuitive interfaces for all platforms

---

**Architecture Version:** 1.0
**Date:** November 21, 2025
**Status:** Production Ready ✅
