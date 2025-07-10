# RuGet - A Simple Rust-Based Downloader

![workflow](https://github.com/greensh16/RuGet/actions/workflows/rust_build.yml/badge.svg)
![workflow](https://github.com/greensh16/RuGet/actions/workflows/rust-clippy.yml/badge.svg)

RuGet is a `wget`/`curl`-like tool written in Rust. It supports parallel downloads, resuming, retries, progress bars, content-disposition filenames, cookie management, and logging.

---

### Basic Usage

```bash
ruget https://example.com
```
Prints response to stdout.

```bash
ruget https://example.com --output page.html
```
Downloads and saves to a file.

---

### Download Multiple URLs

#### From File
```bash
ruget --input urls.txt
```

Where `urls.txt` contains:
```
https://example.com/file1.txt
https://example.com/file2.txt
```

#### Or Inline
```bash
ruget https://site1.com/file1 https://site2.com/file2
```

---

### Save to Directory

```bash
ruget --input urls.txt --output-dir downloads/
```

- Uses `Content-Disposition` or basename from URL for each file
- Ensures output directory exists

---

### Resume & Retry

```bash
ruget --input urls.txt --output-dir downloads --resume --retries 5
```

- `--resume`: continues partially downloaded files  
- `--retries`: retries failed downloads (default: 3)

---

### Cookie Management

RuGet supports wget-compatible cookie handling for session management:

#### Load cookies from a file
```bash
ruget --load-cookies session.txt https://example.com/protected-resource
```

#### Save cookies to maintain sessions
```bash
ruget --save-cookies session.txt https://example.com/login
```

#### Load and save cookies (session persistence)
```bash
ruget --load-cookies session.txt --save-cookies session.txt https://example.com/dashboard
```

#### Include session cookies when saving
```bash
ruget --load-cookies session.txt --save-cookies session.txt --keep-session-cookies https://example.com/api
```

**Cookie File Format**: Uses standard Netscape HTTP Cookie File format, compatible with wget and curl:
```
# Netscape HTTP Cookie File
example.com	TRUE	/	FALSE	0	session_id	abc123
github.com	TRUE	/	TRUE	1735200000	auth_token	def456
```

---

### Progress, Logging, and Control

```bash
ruget --input urls.txt \
      --output-dir downloads \
      --jobs 8 \
      --verbose \
      --log failed.log
```

- `--jobs`: number of parallel downloads (default: CPU count)  
- `--verbose`: print headers for each response  
- `--quiet`: suppress all output except errors  
- `--log`: path to failure log (default: `rustget_failures.log`)

---

### Error Codes

RuGet uses structured error codes to help diagnose issues:

- **E1xx**: I/O errors (file permissions, disk space)
- **E2xx**: HTTP errors (network, server responses)
- **E3xx**: Configuration errors (invalid config values)
- **E4xx**: Network errors (DNS, connectivity)
- **E5xx**: Internal errors (parsing, authentication)

For detailed error descriptions and troubleshooting hints, see [`docs/errors.md`](docs/errors.md).

### Example with Retries and Error Handling

```bash
ruget https://unstable-server.com/large-file.zip \
      --output downloads/file.zip \
      --retries 5 \
      --resume \
      --log-json \
      --log error.log
```

This command will:
- Retry failed downloads up to 5 times with exponential backoff
- Resume partial downloads if the file already exists
- Log structured JSON output for automation parsing
- Save detailed error information to `error.log`

### Complete Example with Session Management

```bash
# Login and save session cookies
ruget --save-cookies session.txt https://secure-site.com/login

# Use saved cookies to access protected resources
ruget --load-cookies session.txt \
      --save-cookies session.txt \
      --keep-session-cookies \
      --output-dir downloads \
      --retries 3 \
      --verbose \
      https://secure-site.com/protected/file1.zip \
      https://secure-site.com/protected/file2.zip
```

This example demonstrates:
- Session cookie persistence across multiple requests
- Authenticated downloads with retry capability
- Multiple file downloads with session maintenance
- Verbose logging for monitoring progress

### JSON Log Output Example

```json
{
  "ts": "2023-09-28T12:34:56Z",
  "level": "ERROR",
  "code": "E201",
  "message": "Connection timeout after 3 retries",
  "context": {
    "url": "https://unstable-server.com/large-file.zip",
    "retry_count": "3",
    "timeout_ms": "30000"
  }
}
```

---

### Full Flag Reference

| Flag                | Description                                      |
|---------------------|--------------------------------------------------|
| `--output <file>`   | Save single URL to a specific file               |
| `--output-dir <dir>`| Save multiple URLs to a directory                |
| `--input <file>`    | Load URLs from a file                            |
| `--header/-H`       | Add custom headers (e.g., `-H "User-Agent: x"`)  |
| `--resume`          | Resume downloads if partially present            |
| `--retries <n>`     | Retry count per URL (default: 3)                 |
| `--jobs <n>`        | Number of concurrent downloads                   |
| `--verbose`         | Print status + headers                           |
| `--quiet`           | Silent except errors                             |
| `--log <file>`      | Log failed downloads (default: `rustget_failures.log`) |
| `--log-json`        | Output logs in JSON format for automation       |
| `--load-cookies <file>` | Load cookies from Netscape-style cookie file |
| `--save-cookies <file>` | Save cookies to file after downloads         |
| `--keep-session-cookies` | Include session cookies when saving         |

---

### Wget Compatibility

RuGet is designed to be a drop-in replacement for common wget usage patterns:

**wget command:**
```bash
wget --load-cookies ~/.session_cookies \
     --save-cookies ~/.session_cookies \
     --keep-session-cookies \
     --output-document=file.html \
     "https://example.com/protected"
```

**equivalent ruget command:**
```bash
ruget --load-cookies ~/.session_cookies \
      --save-cookies ~/.session_cookies \
      --keep-session-cookies \
      --output file.html \
      "https://example.com/protected"
```

- Cookie files are fully compatible between wget and ruget
- Supports the same Netscape HTTP Cookie File format
- Session management works identically
- Easy migration from existing wget scripts

---
