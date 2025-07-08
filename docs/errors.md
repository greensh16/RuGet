# Error Codes Reference

RuGet uses structured error codes to help diagnose and troubleshoot issues. Each error code provides a specific category and detailed guidance for resolution.

## Error Code Categories

### E1xx: I/O Errors
File system and input/output related errors.

| Code | Description | Troubleshooting Hint |
|------|-------------|---------------------|
| E100 | General I/O error | Check file permissions and disk space |
| E101 | File not found | Verify the file path exists |
| E102 | Permission denied | Run with appropriate permissions or check file ownership |
| E103 | Directory creation failed | Check parent directory permissions and disk space |
| E104 | File write error | Ensure sufficient disk space and write permissions |
| E105 | File read error | Check file exists and has read permissions |

### E2xx: HTTP Errors
Network requests and HTTP protocol related errors.

| Code | Description | Troubleshooting Hint |
|------|-------------|---------------------|
| E200 | HTTP request failed | Check URL validity and server status |
| E201 | Connection timeout | Check internet connection or use --timeout option |
| E202 | HTTP client error (4xx) | Verify URL and request parameters |
| E203 | HTTP server error (5xx) | Server is experiencing issues, try again later |
| E204 | Invalid URL | Check URL format and protocol |
| E205 | SSL/TLS error | Check SSL certificate or use --insecure flag |

### E3xx: Configuration Errors
Configuration file and parameter validation errors.

| Code | Description | Troubleshooting Hint |
|------|-------------|---------------------|
| E300 | Configuration error | Check configuration file syntax |
| E301 | Config file not found | Create config file or specify path with --config |
| E302 | Invalid config format | Validate TOML syntax in config file |
| E303 | Missing required config | Add required configuration values |
| E304 | Invalid config value | Check config value format and constraints |

### E4xx: Network Errors
Network connectivity and DNS resolution errors.

| Code | Description | Troubleshooting Hint |
|------|-------------|---------------------|
| E400 | Network error | Check internet connection and network settings |
| E401 | DNS resolution failed | Check DNS settings or use IP address |
| E402 | Network unreachable | Check network connectivity and routing |
| E403 | Connection refused | Check if service is running and accessible |
| E404 | Request timeout | Check internet connection or use --max-retries option |

### E5xx: Internal Errors
Application internal errors and system issues.

| Code | Description | Troubleshooting Hint |
|------|-------------|---------------------|
| E500 | Internal error | Report this issue with debug information |
| E501 | Parse error | Check input format and syntax |
| E502 | Authentication error | Check credentials and authentication method |
| E503 | File system error | Check file system permissions and disk space |
| E504 | Data corruption | Verify file integrity and re-download if needed |
| E505 | Resource exhausted | Free up system resources or increase limits |

## Error Output Formats

### Human-Readable Format
```
[E201][ERROR][2023-09-28T12:34:56Z] Connection timeout after 3 retries | url=https://example.com, retry_count=3
```

### JSON Format (with --log-json)
```json
{
  "ts": "2023-09-28T12:34:56Z",
  "level": "ERROR",
  "code": "E201",
  "message": "Connection timeout after 3 retries",
  "context": {
    "url": "https://example.com",
    "retry_count": "3",
    "timeout_ms": "30000"
  }
}
```

## Common Error Scenarios

### Network Connectivity Issues
- **E201**: Connection timeout - Try increasing timeout or check network
- **E401**: DNS resolution failed - Check DNS settings or use IP address
- **E403**: Connection refused - Verify server is running and accessible

### File System Problems
- **E102**: Permission denied - Check file/directory permissions
- **E103**: Directory creation failed - Verify parent directory permissions
- **E104**: File write error - Check disk space and write permissions

### HTTP-Specific Issues
- **E202**: HTTP 4xx client error - Verify URL and request format
- **E203**: HTTP 5xx server error - Server issue, try again later
- **E205**: SSL/TLS error - Certificate issue, may need --insecure flag

## Best Practices

1. **Check error codes first** - The error code provides the most specific guidance
2. **Use --verbose** for detailed HTTP headers and response information
3. **Enable JSON logging** with --log-json for automated error parsing
4. **Check the log file** - Default location is `rustget_failures.log`
5. **Try increasing retries** - Use --retries for transient network issues

## Related Documentation

- [Retry Mechanisms](retry.md) - Detailed retry behavior and configuration
- [Troubleshooting Guide](troubleshooting.md) - General troubleshooting steps
- [Configuration Reference](../README.md#full-flag-reference) - All command-line options
