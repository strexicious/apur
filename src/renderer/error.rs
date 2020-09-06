pub type Result<T> = std::result::Result<T, APURRendererError>;

#[derive(Debug)]
pub enum APURRendererError {
    BufferDataSizeMismatch,
    BufferTypeInterpretationFailed,
    BufferUsageNotMapRead,
    BufferUsageNotCopyDst,
    BindingResourceTypeUnmatched,
    NumOfBindingsOverflowed,
    NumOfBindingsUnderflowed,
    ErrorOpeningShaderSPV,
}
