/// Extension trait for Result types to provide logging on error
pub trait ResultExt<T, E> {
    /// Log the error if this is an Err, returning the original result
    fn log_err(self) -> Self;
}

impl<T, E: std::fmt::Display> ResultExt<T, E> for Result<T, E> {
    fn log_err(self) -> Self {
        if let Err(ref e) = self {
            tracing::error!("{}", e);
        }
        self
    }
}
