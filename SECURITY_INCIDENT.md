# Security Incident Report - Credential Exposure

**Date**: 2025-11-21  
**Severity**: HIGH  
**Status**: MITIGATED

## Incident Summary

GitGuardian detected exposed Redis URI credentials in the `SuiVerify/suiverify-indexer` repository.

## Affected Credentials

1. **Redis URI**: `redis://default:4OMRhM0XvAVurI2RpVSDKvcvsxMI2s1r@redis-11134.crce182.ap-south-1-1.ec2.cloud.redislabs.com:11134`
2. **PostgreSQL URI**: `postgresql://postgres:suiverifyindexer@db.orjhpozuqydjzqofdkpt.supabase.co:5432/postgres`

## Exposed Files

- `DEPLOYMENT_STATUS.md` (commit: pushed on November 20th 2025, 20:36:44 UTC)
- `SESSION_LOG.md` (contained placeholder text, less critical)
- `architecture.md` (contained placeholder text, less critical)

## Immediate Actions Taken

1. ✅ Removed actual credentials from `DEPLOYMENT_STATUS.md`
2. ✅ Replaced with placeholder values
3. ✅ Committed fix to main branch (commit: 2c8290b)
4. ✅ Verified `.env` is in `.gitignore`

## Required Follow-up Actions

### CRITICAL - Must Do Immediately

1. **Rotate Redis Password**
   - Log into Redis Labs dashboard
   - Change the password for the Redis instance
   - Update the password in:
     - Local `.env` file
     - Render environment variables
     - Any other services using this Redis instance

2. **Rotate Supabase Password**
   - Log into Supabase dashboard
   - Change the `postgres` user password
   - Update the password in:
     - Local `.env` file
     - Render environment variables
     - Any other services using this database

3. **Review Git History**
   - The credentials are still in Git history
   - Consider using `git filter-branch` or `BFG Repo-Cleaner` to remove from history
   - Or accept that the credentials are compromised and rotation is sufficient

### Recommended Additional Actions

4. **Enable IP Whitelisting**
   - Configure Redis Labs to only accept connections from known IPs
   - Configure Supabase to only accept connections from known IPs

5. **Monitor for Unauthorized Access**
   - Check Redis Labs logs for suspicious activity
   - Check Supabase logs for suspicious activity
   - Monitor for unusual database queries or data access

6. **Implement Secrets Management**
   - Consider using a secrets manager (e.g., HashiCorp Vault, AWS Secrets Manager)
   - Use environment variables exclusively for sensitive data
   - Never commit credentials to documentation files

## Prevention Measures

1. ✅ `.env` files are already in `.gitignore`
2. ⚠️ Add pre-commit hooks to scan for credentials
3. ⚠️ Use `.env.example` with placeholder values only
4. ⚠️ Review all markdown files before committing
5. ⚠️ Enable GitGuardian or similar tools for automatic scanning

## Lessons Learned

- Documentation files (`.md`) can accidentally contain real credentials
- Always use placeholders like `YOUR_PASSWORD` in documentation
- Credentials should ONLY exist in:
  - Local `.env` files (gitignored)
  - Environment variables in deployment platforms
  - Secure secrets managers

## Timeline

- **2025-11-20 20:36:44 UTC**: Credentials pushed to GitHub
- **2025-11-21 02:52:36 IST**: GitGuardian alert received
- **2025-11-21 02:55:00 IST**: Credentials removed from documentation
- **2025-11-21 02:55:30 IST**: Fix pushed to main branch

## Status

- [x] Credentials removed from current codebase
- [ ] **CRITICAL**: Rotate Redis password
- [ ] **CRITICAL**: Rotate Supabase password
- [ ] Review Git history
- [ ] Enable IP whitelisting
- [ ] Monitor for unauthorized access
