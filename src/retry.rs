use std::time::Duration;
use tokio::time::sleep;

/// Retry an async operation with exponential backoff.
///
/// Retries up to `max_retries` times when the error is transient
/// (network errors or HTTP 5xx). The initial delay is 500 ms and doubles
/// after each failed attempt.
pub async fn with_retry<F, Fut, T>(max_retries: u32, mut f: F) -> Result<T, crate::error::InfsError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, crate::error::InfsError>>,
{
    let mut delay = Duration::from_millis(500);
    for attempt in 0..=max_retries {
        match f().await {
            Ok(v) => return Ok(v),
            Err(e) => {
                if !e.is_transient() || attempt == max_retries {
                    return Err(e);
                }
                tracing::debug!(
                    "Attempt {} failed ({}), retrying in {:?}",
                    attempt + 1,
                    e,
                    delay
                );
                sleep(delay).await;
                delay = delay.saturating_mul(2);
            }
        }
    }
    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::InfsError;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_retry_succeeds_on_first_try() {
        let calls = Arc::new(AtomicU32::new(0));
        let calls_clone = calls.clone();
        let result: Result<u32, InfsError> = with_retry(3, || {
            let c = calls_clone.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                Ok(42u32)
            }
        })
        .await;
        assert_eq!(result.unwrap(), 42);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_does_not_retry_non_transient_errors() {
        let calls = Arc::new(AtomicU32::new(0));
        let calls_clone = calls.clone();
        let result: Result<u32, InfsError> = with_retry(3, || {
            let c = calls_clone.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                Err(InfsError::ProviderNotConfigured("test".to_string()))
            }
        })
        .await;
        assert!(result.is_err());
        // Should only try once since the error is not transient
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
