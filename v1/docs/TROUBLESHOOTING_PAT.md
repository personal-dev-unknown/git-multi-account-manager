# PAT Token Troubleshooting Guide

**Date:** November 22, 2025

---

## Issue: 401 Unauthorized Error

### Error Message
```
✗ Failed to fetch repositories: Failed to fetch GitHub repositories: 401 Client Error: Unauthorized for url: https://api.github.com/user/repos?per_page=100&page=1&sort=updated&type=all
```

### What This Means
The PAT token you provided is **invalid, expired, or doesn't have the required scopes**.

---

## Solution: Generate a New PAT Token

### For GitHub

1. Go to: https://github.com/settings/tokens
2. Click **"Generate new token"** → **"Generate new token (classic)"**
3. Fill in the form:
   - **Token name:** `Git Manager CLI`
   - **Expiration:** Select "90 days" or "No expiration"
   - **Scopes:** Check these boxes:
     - ✅ `repo` (Full control of private repositories)
     - ✅ `read:user` (Read user profile data)
4. Click **"Generate token"**
5. **COPY THE TOKEN IMMEDIATELY** (you won't see it again!)
6. Paste it when prompted in Git Manager

### For GitLab

1. Go to: https://gitlab.com/-/profile/personal_access_tokens
2. Fill in the form:
   - **Token name:** `Git Manager CLI`
   - **Expiration date:** Select a date 90 days from now
   - **Scopes:** Check these boxes:
     - ✅ `api` (Read/write API)
     - ✅ `read_api` (Read API)
     - ✅ `read_repository` (Read repository)
3. Click **"Create personal access token"**
4. **COPY THE TOKEN IMMEDIATELY** (you won't see it again!)
5. Paste it when prompted in Git Manager

### For Bitbucket

1. Go to: https://bitbucket.org/account/settings/app-passwords/new
2. Fill in the form:
   - **Label:** `Git Manager CLI`
   - **Permissions:** Check:
     - ✅ `repositories:read` (Read repositories)
3. Click **"Create"**
4. **COPY THE PASSWORD IMMEDIATELY** (you won't see it again!)
5. Paste it when prompted in Git Manager

---

## How Git Manager Will Handle Your PAT

### When You First Enter a PAT

```
⚠️ Account 'devonionMoses' has no Personal Access Token (PAT)
To fetch your repositories, you need a PAT from GITHUB

How to get a PAT:
1. Go to: https://github.com/settings/tokens
2. Click 'Generate new token' → 'Generate new token (classic)'
3. Select scopes: repo, read:user
4. Click 'Generate token' and copy it

Enter your Personal Access Token: ••••••••••••••••••••••••••••••••••••••••••••••

⏳ Testing Personal Access Token...
✓ PAT is valid!

✓ PAT saved for account 'devonionMoses'
```

### If Your PAT is Invalid

```
⏳ Testing Personal Access Token...
⚠️ PAT is invalid. Try again (1/3)

Enter your Personal Access Token: ••••••••••••••••••••••••••••••••••••••••••••••

⏳ Testing Personal Access Token...
✓ PAT is valid!

✓ PAT saved for account 'devonionMoses'
```

### If Your PAT Expires Later

```
Step 3: Fetching Your Repositories...
✗ Failed to fetch repositories: 401 Unauthorized

⚠️ Invalid or expired Personal Access Token
The PAT for account 'devonionMoses' is no longer valid

How to get a new PAT:
[Platform-specific instructions]

Enter your new Personal Access Token: ••••••••••••••••••••••••••••••••••••••••••••••

⏳ Testing Personal Access Token...
✓ PAT is valid!

✓ PAT updated for account 'devonionMoses'

Step 3: Fetching Your Repositories...
✓ Found 15 repositories
```

---

## Common Issues

### Issue 1: "PAT is invalid. Try again (1/3)"

**Cause:** You made a typo when copying the PAT

**Solution:**
1. Go back to GitHub/GitLab/Bitbucket
2. Copy the PAT again (carefully!)
3. Paste it when prompted
4. You get 3 attempts before it exits

### Issue 2: "Failed to validate PAT after 3 attempts"

**Cause:** The PAT you're entering is consistently invalid

**Solution:**
1. Generate a NEW PAT (the old one might be revoked)
2. Make sure you copied it correctly
3. Check that you selected the right scopes
4. Try again

### Issue 3: "Invalid or expired Personal Access Token" (401 Error)

**Cause:** The PAT was valid before but is now expired or revoked

**Solution:**
1. Generate a NEW PAT
2. Enter it when prompted
3. System will test it and save it
4. You get 3 attempts

### Issue 4: "PAT is required to fetch repositories"

**Cause:** You didn't enter a PAT (pressed Enter without typing)

**Solution:**
1. You must enter a PAT to fetch repositories
2. Go back and select the account again
3. Enter a valid PAT this time

---

## How to Verify Your PAT Works

### Using curl (Command Line)

**GitHub:**
```bash
curl -H "Authorization: token YOUR_PAT_HERE" \
  https://api.github.com/user
```

**GitLab:**
```bash
curl -H "PRIVATE-TOKEN: YOUR_PAT_HERE" \
  https://gitlab.com/api/v4/user
```

**Bitbucket:**
```bash
curl -u "x-token-auth:YOUR_PAT_HERE" \
  https://api.bitbucket.org/2.0/user
```

If you get a 200 response, your PAT is valid!

### Using Git Manager

Git Manager will automatically test your PAT when you enter it:

```
⏳ Testing Personal Access Token...
✓ PAT is valid!
```

If you see this message, your PAT is working!

---

## PAT Scopes Explained

### GitHub Scopes

- **`repo`** - Full control of private repositories (required)
- **`read:user`** - Read user profile data (required)
- **`workflow`** - Update GitHub Actions workflows (optional)
- **`gist`** - Create gists (optional)

### GitLab Scopes

- **`api`** - Read/write API (required)
- **`read_api`** - Read API (required)
- **`read_repository`** - Read repository (required)
- **`write_repository`** - Write repository (optional)

### Bitbucket Scopes

- **`repositories:read`** - Read repositories (required)
- **`repositories:write`** - Write repositories (optional)
- **`account:read`** - Read account info (optional)

---

## Security Best Practices

✅ **DO:**
- Generate a new PAT for each application
- Use meaningful names (e.g., "Git Manager CLI")
- Set an expiration date (90 days recommended)
- Copy the PAT immediately (you can't see it again)
- Store it securely (Git Manager stores it in ~/.config/git-manager/)

❌ **DON'T:**
- Share your PAT with anyone
- Commit your PAT to git
- Use the same PAT for multiple applications
- Set expiration to "No expiration" (unless necessary)
- Paste your PAT in chat or email

---

## Still Having Issues?

1. **Clear the old PAT:**
   ```bash
   # Edit ~/.config/git-manager/accounts.json
   # Remove the "pat_token" field for your account
   ```

2. **Generate a fresh PAT:**
   - Go to GitHub/GitLab/Bitbucket
   - Create a new token
   - Make sure you have the right scopes

3. **Test with Git Manager:**
   - Select your account
   - Enter the new PAT
   - System will test it automatically

4. **If still failing:**
   - Check that your PAT has the right scopes
   - Check that your PAT hasn't expired
   - Check that your PAT hasn't been revoked
   - Try generating a completely new PAT

---

**Status:** ✅ TROUBLESHOOTING GUIDE COMPLETE
