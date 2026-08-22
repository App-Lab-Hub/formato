# Security Policy

## Threat Model

### Overview

Formato is a cross-platform desktop application (macOS, Windows, Linux) built on Rust and Tauri. It processes user-supplied files (text, documents, images, audio, video) locally, uses FFmpeg and LibreOffice for media conversion, and integrates with local speech models (Whisper, Piper). No data is uploaded to the internet.

### Assets

| Asset                          | Description                                                                          |
| ------------------------------ | ------------------------------------------------------------------------------------ |
| User files                     | Text, documents, images, audio, and video files processed locally                     |
| Speech models                  | Local models (Whisper, Piper) used for speech synthesis and recognition               |
| User preferences & settings    | Theme, language, caching, and archiving preferences                                   |
| Conversion history             | Database of converted files and metadata stored locally                               |

### Threat Actors

| Actor                   | Motivation                                                 |
| ----------------------- | ---------------------------------------------------------- |
| Malicious file author  | Craft a malformed file to exploit the parser or converter  |
| Compromised dependency | Supply chain attack via npm or Cargo ecosystem             |
| Local attacker         | Access another user's files or settings on a shared device |

### Attack Surfaces & Mitigations

#### 1. File Parsing & Conversion

- **Risk:** Malformed files could trigger parser bugs, path traversal, or memory corruption during conversion.
- **Mitigations:** Files are processed in isolated Rust modules. Paths are validated to prevent traversal. FFmpeg and LibreOffice are used as external processes with restricted arguments.

#### 2. Speech Model Loading

- **Risk:** Malicious or tampered model files could introduce code execution or resource exhaustion.
- **Mitigations:** Models are downloaded from official sources and stored in a local directory. Paths are validated before loading.

#### 3. Local Database

- **Risk:** SQLite database could be corrupted or accessed by unauthorized local processes.
- **Mitigations:** Database is stored in the application data directory. Access is restricted to the application process.

#### 4. Supply Chain

- **Risk:** Compromised npm or Cargo packages could introduce malicious code.
- **Mitigations:** Dependencies are pinned via `package-lock.json` and `Cargo.lock`. Dependabot is enabled for automated vulnerability detection.

#### 5. Desktop Native Code (Tauri)

- **Risk:** Tauri IPC commands could be abused by malicious web content to access the filesystem or OS APIs.
- **Mitigations:** Tauri's allowlist restricts which IPC commands are exposed. File system access is scoped to the application data directory.

### Out of Scope

- Vulnerabilities in user's operating system or browser outside of Formato's control
- Physical access attacks to a user's device
- Issues in third-party services (FFmpeg, LibreOffice, GitHub, Boosty) themselves

---

## Supported Versions

Formato is currently in its first release (version 1.0.0). We are actively developing the app and will provide security updates for the latest stable version. Older versions may not receive patches as we continue to iterate.

---

## Reporting a Vulnerability

If you discover a potential security issue in Formato, please **do not** open a public issue. We want to fix it before it is publicly known.

You can report a vulnerability by:

1. **Opening a private Security Advisory** on GitHub:
   - Go to the repository: `https://github.com/App-Lab-Hub/formato`
   - Click on **"Security"** tab
   - Click on **"Report a vulnerability"**
   - Fill out the form with details about the vulnerability

Please include the following information in your report:
- A description of the vulnerability
- Steps to reproduce it
- The affected version(s)
- Any potential impact

---

## What to Expect

We will:
- Acknowledge your report as soon as possible
- Investigate the issue and determine the severity
- Do our best to fix the issue and release a patch
- Credit you in the release notes (if you wish)

**Please note:** Since Formato is a small, community-driven project, we cannot guarantee a specific timeline for fixes. However, we take security seriously and will prioritize critical issues.

---

## Incident Response Plan

When a security vulnerability is confirmed, we follow this simplified process:

### 1. Triage

- Assign a severity level (Critical / High / Medium / Low) based on impact and exploitability.
- Identify affected versions and components.

### 2. Containment

- Assess whether an immediate mitigation or workaround can be published.
- Limit further exposure where possible (e.g., disable affected features, update dependencies).

### 3. Remediation

- Develop and internally review a fix.
- Validate the fix does not introduce regressions.
- Prepare a patched release.

### 4. Disclosure & Release

- Coordinate disclosure timing with the reporter.
- Publish a GitHub Security Advisory with CVE if applicable.
- Release the patched version and notify users via release notes.

### 5. Post-Incident Review

- Document the root cause, timeline, and resolution.
- Update processes or controls to prevent recurrence.

---

## Security Best Practices

Formato processes all data locally and does not upload files to the internet. However, we still encourage users to:
- Keep the app updated to the latest version
- Only download the app from official sources (GitHub Releases, Boosty, etc.)
- Be cautious when opening files from untrusted sources
