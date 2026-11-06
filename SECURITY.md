# Security policy

## Supported versions

No version is currently supported for production security. This is an unfinished research snapshot with known defects; see [RESEARCH_STATUS.md](RESEARCH_STATUS.md) and the independent audits. Do not connect valuable accounts or confidential data merely because a component test or packaging scan passed. No patch schedule, response SLA or security-support promise is made.

## Reporting a vulnerability

The selected channel is **GitHub private vulnerability reporting** for the eventual public repository: open its **Security → Advisories → Report a vulnerability** form. Use it only when that button is available. The repository owner must enable this feature in repository settings; a local SECURITY.md file cannot activate it. It has not been enabled by this packaging task because no public repository has been created.

If private reporting is unavailable, do not post exploit details, live credentials or personal data in a public issue or on X. You may open a minimal issue requesting a private channel without disclosing sensitive details. The public author profile is [@ignatXfire](https://x.com/ignatXfire); it is not a guaranteed private or monitored channel. Do not assume X direct messages are enabled. For an exposed provider credential, contact the provider and revoke it yourself rather than waiting for a project response.

A useful report gives the exact revision/component, a synthetic reproduction, observed versus expected behavior, impact and evidence boundaries. Do not test on other people's systems or perform real external effects without their authorization.

## Packaging is not product security

The public distribution omits the original Git history, private conversations, runtime credentials/databases and private operational state. Scans reduce accidental disclosure; they cannot establish that no secret or vulnerability exists. This policy does not make a legal safe-harbor promise or authorize testing of third-party systems.
