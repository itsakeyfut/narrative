//! Framework error types

use thiserror::Error;

/// Framework error type
#[derive(Debug, Error)]
pub enum FrameworkError {
    #[error("Window creation failed: {0}")]
    WindowCreation(String),

    #[error("GPU initialization failed: {0}")]
    GpuInit(String),

    #[error("Surface configuration failed: {0}")]
    SurfaceConfig(String),

    #[error("Render error: {0}")]
    Render(String),

    #[error("Layout error: {0}")]
    Layout(String),

    #[error("Event loop error: {0}")]
    EventLoop(String),

    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),
}

impl From<wgpu::CreateSurfaceError> for FrameworkError {
    fn from(err: wgpu::CreateSurfaceError) -> Self {
        FrameworkError::GpuInit(err.to_string())
    }
}

impl From<wgpu::RequestDeviceError> for FrameworkError {
    fn from(err: wgpu::RequestDeviceError) -> Self {
        FrameworkError::GpuInit(err.to_string())
    }
}

impl From<wgpu::SurfaceError> for FrameworkError {
    fn from(err: wgpu::SurfaceError) -> Self {
        FrameworkError::Render(err.to_string())
    }
}

/// Result type for framework operations
pub type FrameworkResult<T> = Result<T, FrameworkError>;
