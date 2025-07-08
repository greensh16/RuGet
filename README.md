# RuGet - A Simple Rust-Based Downloader

![workflow](https://github.com/greensh16/RuGet/actions/workflows/rust_build.yml/badge.svg)
![workflow](https://github.com/greensh16/RuGet/actions/workflows/rust-clippy.yml/badge.svg)

RuGet is a `wget`/`curl`-like tool written in Rust. It supports parallel downloads, resuming, retries, progress bars, content-disposition filenames, and logging.

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

---
