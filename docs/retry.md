# Retry Mechanisms in RuGet

RuGet provides robust support for retrying network operations to increase download reliability. This document outlines the configuration and behavior of retry operations for different scenarios.

## Configurable Parameters

- `--retries <n>`: Set the maximum number of retry attempts (default is 3).
- `--timeout <ms>`: Set the timeout for each request (in milliseconds).
- `--backoff-base <ms>`: Initial backoff duration, in milliseconds.
- `--backoff-factor <n>`: Exponential backoff factor (default is 2.0).
- `--max-backoff <ms>`: Maximum backoff duration, in milliseconds.

## Retry Logic

The retry mechanism in RuGet uses an exponential backoff strategy with jitter to reduce the likelihood of further amplifying network congestion.

### Exponential Backoff

Retry attempts are spaced out with increasing durations:

- **First retry**: base backoff duration
- **Subsequent retries**: `base * factor^attempt_number`

With `--backoff-base` set to 100ms and a factor of 2, retries are scheduled at 100ms, 200ms, 400ms, etc.

### Jitter

To avoid thundering herd problems (many retries hitting at once), RuGet includes random jitter to spread out retry timing.

## Network Errors Considered for Retries

By default, all network-related errors will trigger a retry, unless:

1. An `HTTP 4xx` client error indicates a bad request.
2. The maximum retry count is reached.

## Example Usage

### CLI Command with Retries and Backoff

```bash
ruget https://example.com/large-file.zip \
      --output downloaded/large-file.zip \
      --retries 5 \
      --timeout 30000 \
      --backoff-base 250 \
      --backoff-factor 3
```

This command retries downloads up to 5 times, with a 30-second timeout per attempt, starting the retry delay at 250ms and increasing by a factor of 3 for each attempt.

## Logging and Insights

Retries, timeouts, and errors are logged to provide transparency into network operations. Enable `--verbose`  and `--log-json` for deeper insights:

- Retried attempts are indicated in logs with specific retry count and delay.
- JSON formatted logs provide structured data ideal for integrations.

```json
{
  "ts": "2023-09-28T12:34:56Z",
  "level": "INFO",
  "message": "Retry attempt",
  "context": {
    "url": "https://example.com/large-file.zip",
    "attempt": "2",
    "next_delay_ms": "750"
  }
}
```

## Best Practices

1. **Set appropriate timeouts** based on network conditions to balance between retries and responsiveness.
2. **Tune backoff and retry parameters** for high reliability, especially with unstable connections.
3. **Use structured logs** for monitoring and alerting purposes.

## Related Documentation

- [Error Codes Reference](errors.md) - Understand error classifications and handling
- [Structured Logging](json-logging.md) - Learn how to enable JSON formatted logs
- [Configuration Reference](../README.md#full-flag-reference) - Explore all configuration options
