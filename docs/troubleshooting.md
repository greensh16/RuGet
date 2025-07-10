# Troubleshooting Guide

This document provides troubleshooting information for common RuGet errors. Each error code includes a description, possible causes, and suggested solutions.

## Error Code Reference

### I/O Errors (E1xx)

| Error Code | Message | Description | Troubleshooting Steps |
|------------|---------|-------------|----------------------|
| E100 | General I/O error | A generic input/output operation failed | Check file permissions and disk space |
| E101 | File not found | The specified file could not be located | Verify the file path exists |
| E102 | Permission denied | Insufficient permissions to access file/directory | Run with appropriate permissions or check file ownership |
| E103 | Directory creation failed | Unable to create required directory | Check parent directory permissions and disk space |
| E104 | File write error | Failed to write data to file | Ensure sufficient disk space and write permissions |
| E105 | File read error | Failed to read data from file | Check file exists and has read permissions |

### HTTP Errors (E2xx)

| Error Code | Message | Description | Troubleshooting Steps |
|------------|---------|-------------|----------------------|
| E200 | HTTP request failed | General HTTP request failure | Check URL validity and server status |
| E201 | Connection timeout | Request timed out waiting for server response | Check internet connection or use `--timeout` option |
| E202 | HTTP client error | Server returned 4xx error (client error) | Verify URL and request parameters |
| E203 | HTTP server error | Server returned 5xx error (server error) | Server is experiencing issues, try again later |
| E204 | Invalid URL | The provided URL is malformed or invalid | Check URL format and protocol |
| E205 | SSL/TLS error | SSL/TLS connection failed | Check SSL certificate or use `--insecure` flag |

### Configuration Errors (E3xx)

| Error Code | Message | Description | Troubleshooting Steps |
|------------|---------|-------------|----------------------|
| E300 | Configuration error | General configuration problem | Check configuration file syntax |
| E301 | Config file not found | Configuration file doesn't exist | Create config file or specify path with `--config` |
| E302 | Invalid config format | Configuration file has syntax errors | Validate TOML syntax in config file |
| E303 | Missing required config | Required configuration values are missing | Add required configuration values |
| E304 | Invalid config value | Configuration value is invalid or out of range | Check config value format and constraints |

### Network Errors (E4xx)

| Error Code | Message | Description | Troubleshooting Steps |
|------------|---------|-------------|----------------------|
| E400 | Network error | General network connectivity issue | Check internet connection and network settings |
| E401 | DNS resolution failed | Unable to resolve hostname to IP address | Check DNS settings or use IP address |
| E402 | Network unreachable | Network destination is unreachable | Check network connectivity and routing |
| E403 | Connection refused | Server refused the connection | Check if service is running and accessible |
| E404 | Request timeout | Network request timed out | Check internet connection or use `--max-retries` option |

### Internal Errors (E5xx)

| Error Code | Message | Description | Troubleshooting Steps |
|------------|---------|-------------|----------------------|
| E500 | Internal error | Unexpected internal application error | Report this issue with debug information |
| E501 | Parse error | Failed to parse input data | Check input format and syntax |
| E502 | Authentication error | Authentication or authorization failed | Check credentials and authentication method |
| E503 | File system error | File system operation failed | Check file system permissions and disk space |
| E504 | Data corruption | Data integrity check failed | Verify file integrity and re-download if needed |
| E505 | Resource exhausted | System resources (memory, handles) exhausted | Free up system resources or increase limits |

## Common Solutions

### Connection Issues
1. **Check Network Connectivity**: Ensure you have an active internet connection
2. **Use Retries**: Add `--max-retries 5` to retry failed requests
3. **Increase Timeout**: Use `--timeout 60` to allow more time for slow connections
4. **Check Firewall**: Ensure RuGet is allowed through your firewall

### Permission Issues
1. **Run as Administrator/Root**: Use `sudo` on Unix systems or run as Administrator on Windows
2. **Check File Ownership**: Ensure you own the target directory: `chown -R $USER:$USER /path/to/directory`
3. **Set Proper Permissions**: Make directories writable: `chmod 755 /path/to/directory`

### SSL/TLS Issues
1. **Update CA Certificates**: Ensure your system's CA certificates are up to date
2. **Use Insecure Flag**: For testing only, use `--insecure` to skip certificate verification
3. **Check System Time**: Ensure your system clock is accurate

### Configuration Issues
1. **Validate TOML Syntax**: Use an online TOML validator to check your config file
2. **Check File Location**: Ensure config file is in the expected location (`~/.ruget/config.toml`)
3. **Review Default Values**: Check the default configuration for required fields

## Getting More Help

### Debug Information
Run RuGet with verbose logging to get more detailed error information:
```bash
ruget --verbose <url>
```

### Reporting Issues
When reporting issues, please include:
- The complete error message including error code
- The command you ran
- Your operating system and RuGet version
- Any relevant configuration files (with sensitive data removed)

### Support Channels
- GitHub Issues: [https://github.com/username/ruget/issues](https://github.com/username/ruget/issues)
- Documentation: Check the README.md for additional configuration options
