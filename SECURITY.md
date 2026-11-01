# Security policy

WarCommand Tracker has no accounts and collects no personal data (see [PRIVACY.md](PRIVACY.md)). What needs
protecting is the update path to installed apps, the collector's secret key, and the shared war database.

## Supported versions

Only the [latest release](https://github.com/zearkreast/WarRoom/releases/latest) gets security fixes. Installed
apps offer each new release themselves, and the website always runs the newest `main`. Older versions and
pre-releases are not patched.

## Reporting a vulnerability

Please report it privately, not in a public issue: use
[Report a vulnerability](https://github.com/zearkreast/WarRoom/security/advisories/new) on the repository's
Security tab. Include what you found, the version and platform, and the steps to reproduce it.

This is a small fan project, so there is no bounty and no guaranteed response time. Reports are looked at as soon
as possible. Once a fix is out the issue is published as a security advisory, and you are credited if you want to
be.

## In scope

- The desktop and Android apps, including how they download and install updates.
- The website.
- The collector.
- The Supabase schema in `supabase/migrations`, for example writing to the database or reading anything beyond the
  published war data with the publishable key.
- The GitHub workflows and how releases are signed.

## Not vulnerabilities

- **The Supabase publishable key in the app.** It is public by design and row-level security leaves it read-only.
  It only matters if you can use it to do more than that.
- **Wrong or missing war data.** Open a normal issue.
- **Problems in services the project relies on**, such as the game's API, the community API, Supabase, GitHub or
  Firebase. Report those to their owners.

## Testing

Please test against your own copy: run the migrations on your own Supabase project and point the app at it with
`WARROOM_SUPABASE_URL` and `WARROOM_SUPABASE_PUBLISHABLE_KEY` (see `.env.example`). Don't load-test, scan or try to
write to the live database, since it runs on a small free plan that every player shares.

## How updates are protected

Desktop updates are signed, and installed apps check them against the public key built into the app. Android APKs
are signed with the project's key, and Android refuses to install an update signed with a different one. If you
find a way around either check, that is the kind of report this policy is most interested in.
