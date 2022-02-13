use std::{ops::Deref, sync::Arc, sync::RwLock, collections::HashMap, str::FromStr};
use bevy::prelude::{Stage, World};
use tokio::sync::oneshot::{error::TryRecvError, channel};

use crate::errors::Result;

pub mod wordcloud;

/// A wrapper for controlling 3D visuals.
/// This is safe to share across threads.
#[derive(Debug, Clone)]
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

    // Get the currently running visual.
    // Don't keep this too long,
    // since it keeps the visual from being changed while you're holding it.
    // pub fn current(&self) -> impl Deref<Target=Box<dyn Visual>> + '_ {
    //     &self.0.deref().read().unwrap().current
    // }
}

#[derive(Debug)]
pub struct VisualManagerInner {
    current: Box<dyn Visual>,
    control: RemoteControl,
}

impl VisualManagerInner {
    fn new() -> Result<Self> {
        let control = RemoteControl::default();
        let current = Visuals::default().create();
        let offswitch=control.get_sender();
        current.start(control)?;
        Ok(Self { current, offswitch })
    }

    /// Change the current visual
    /// This is slow and blocking, but realistically, you can only do this once every few seconds anyway
    pub async fn switch_mut(&mut self, vis: Visuals) -> Result<()> {
        let (control, stage) = new_remote();
        offswitch.send(Ok(()));
        self.current = vis.create();
        self.offswitch = control.get_sender();
        self.current.start(control);
        Ok(())
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

/// An action and it's associated reply box
#[derive(Debug)]
pub struct VisualMessage {
    reply: OneshotSender,
    action: VisualAction
}

/// A data visualization application
pub trait Visual : std::fmt::Debug + Send + Sync {
    /// Start the application
    fn start(&self, control: RemoteControl) -> Result<()>;
    /// Respond to live changes to the application, usually from the webserver
    fn react(&self, action: VisualAction) -> OneshotReceiver;
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
    async fn quit(&self) {
        self.0.outbound.send(());
        self.0.inbound.recv_async()
    }
}
struct RemoteStage(Link<()>);
fn new_remote() -> (RemoteControl, RemoteStage) {
    let (left, right) = Link::new();
    (RemoteControl(left), RemoteStage(right))
}
impl Stage for RemoteControl {
    fn run(&mut self, world: &mut World) {
        for msg in self.0.inbound.try_iter() {
            world.clear_entities();
            // Send back that the world is cleared
            self.0.outbound.send(()).unwrap_or_default();
        }
    }
}