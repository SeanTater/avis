use std::{sync::Arc, sync::RwLock, str::FromStr};
use bevy::prelude::{Stage, World};

use crate::errors::Result;

pub mod wordcloud;

/// A wrapper for controlling 3D visuals.
/// This is safe to share across threads.
#[derive(Clone)]
pub struct VisualManager(Arc<RwLock<VisualManagerInner>>);
impl VisualManager {
    /// Create a new visual manager.
    pub fn new() -> Result<VisualManager> {
        Ok(Self(Arc::new(RwLock::new(VisualManagerInner::new()?))))
    }

    /// Change the current visual
    /// This is slow and blocking, but realistically, you can only do this once every few seconds anyway
    pub async fn switch(&self, vis: Visuals) -> Result<()> {
        self.0.write().unwrap().switch_mut(vis).await
    }

    /// Run the whole visual, blocking forever.
    pub fn run(&self) -> Result<()> {
        self.0.write().unwrap().start()
    }
}

pub struct VisualManagerInner {
    current: Box<dyn Visual>,
    control: RemoteControl,
    stage: Option<RemoteStage>
}

impl VisualManagerInner {
    fn new() -> Result<Self> {
        let (control, stage) = new_remote();
        let current = Visuals::default().create();
        Ok(Self { current, control, stage: Some(stage) })
    }

    /// Change the current visual
    /// This is slow and blocking, but realistically, you can only do this once every few seconds anyway
    pub async fn switch_mut(&mut self, vis: Visuals) -> Result<()> {
        let (control, stage) = new_remote();
        control.quit().await;
        self.control = control;
        self.current = vis.create();
        self.stage = Some(stage);
        Ok(())
    }

    pub fn start(&mut self) -> Result<()> {
        self.stage.take().map(|stage| self.current.start(stage)).unwrap_or(Ok(()))
    }
}

/// Anything a visual can do
#[derive(Debug, Clone)]
pub enum VisualAction {
    AddWord(String)
}

/// A sender, for when a message is received and the result needs to be send back to the caller
pub type OneshotSender = tokio::sync::oneshot::Sender<Result<()>>;
/// A receiver, for when you need to be notified that the result is ready
pub type OneshotReceiver = tokio::sync::oneshot::Receiver<Result<()>>;

/// A data visualization application
pub trait Visual : std::fmt::Debug + Send + Sync {
    /// Start the application
    fn start(&self, remote_stage: RemoteStage) -> Result<()>;
}

#[derive(Debug, Clone)]
pub enum Visuals {
    WordCloud
}
impl Visuals {
    pub fn create(&self) -> Box<dyn Visual> {
        Box::new(match self {
            Self::WordCloud => wordcloud::WordCloudVisual::new()
        })
    }
}
impl Default for Visuals {
    fn default() -> Self {
        Self::WordCloud
    }
}
impl FromStr for Visuals {
    type Err = ();
    fn from_str(text: &str) -> std::result::Result<Self, ()> {
        match text {
            "WordCloud" => Ok(Self::WordCloud),
            _ => Err(())
        }
    }
}

/// Remote procedure calls for communicating between Actix and Bevy event loops
#[derive(Debug)]
pub struct Link<X: Send + Sync> {
    inbound: flume::Receiver<X>,
    outbound: flume::Sender<X>
}
impl<X: Send+Sync> Link<X> {
    pub fn new() -> (Self, Self) {
        let rightward = flume::bounded(5);
        let leftward = flume::bounded(5);
        (Self {
            outbound: rightward.0,
            inbound: leftward.1,
        },Self {
            outbound: leftward.0,
            inbound: rightward.1,
        },)
    }
}

/// A stage for a Bevy pipeline, allowing control from outside
struct RemoteControl(Link<()>);
impl RemoteControl {
    /// Quit the visual, and wait for confirmation it has finished.
    async fn quit(&self) {
        if let Ok(_) = self.0.outbound.send(()) {
            // Failure means the remote died, which is actually success this time
            self.0.inbound.recv_async().await.unwrap_or_default()
        }
        // Too many quits, we must already be quitting, so no error
    }
}
pub struct RemoteStage(Link<()>);
fn new_remote() -> (RemoteControl, RemoteStage) {
    let (left, right) = Link::new();
    (RemoteControl(left), RemoteStage(right))
}
impl Stage for RemoteStage {
    fn run(&mut self, world: &mut World) {
        for _msg in self.0.inbound.try_iter() {
            world.clear_entities();
            // Send back that the world is cleared
            self.0.outbound.send(()).unwrap_or_default();
        }
    }
}