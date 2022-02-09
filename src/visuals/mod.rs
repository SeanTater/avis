use std::{ops::Deref, sync::Arc};

pub mod wordcloud;

/// A wrapper for controlling 3D visuals.
/// This is safe to share across threads.
#[derive(Debug, Clone)]
pub struct VisualManager(Arc<VisualManagerInner>);
impl VisualManager {
    /// Create a new visual manager.
    pub fn new() -> VisualManager {
        Self(Arc::new(VisualManagerInner {
            current: wordcloud::WordCloudVisual::new(),
        }))
    }
}
impl Deref for VisualManager {
    type Target = VisualManagerInner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct VisualManagerInner {
    current: wordcloud::WordCloudVisual,
}
impl VisualManagerInner {
    /// Get the currently running visual
    pub fn current(&self) -> &wordcloud::WordCloudVisual {
        &self.current
    }
}

/// Anything a visual can do
pub enum VisualAction {
    AddWord(String),
    Exit,
}
