use crate::tui::TuiWidget;
use crossterm::event::KeyEvent;
use ratatui::prelude::{Frame, Rect};

pub trait TabContent: 'static {
    type Key: for<'a> TryFrom<&'a KeyEvent, Error = ()>;
    type State;

    const TITLE: &str;

    fn handle_key_event(
        &mut self,
        key: Self::Key,
        task_set: &mut FutureSet<Self>,
        state: &mut Self::State,
    );

    fn render(&self, f: &mut Frame, area: Rect, state: &mut Self::State);
}

type CallBack<C> = Box<dyn FnOnce(&mut C) + Send>;
pub type FutureSet<C> = tokio::task::JoinSet<CallBack<C>>;

pub trait FutureSetExt<C>: Future<Output = CallBack<C>>
where
    Self: Sized + Send + 'static,
    C: 'static,
{
    fn spawn(self, set: &mut FutureSet<C>) {
        set.spawn(self);
    }
}
impl<F, C> FutureSetExt<C> for F
where
    F: Future<Output = CallBack<C>> + Send + 'static,
    C: 'static,
{
}

pub struct Tab<C: TabContent> {
    content: C,
    state: C::State,
    tasks: FutureSet<C>,
}

impl<C> Tab<C>
where
    C: TabContent,
{
    pub fn content_mut(&mut self) -> &mut C {
        &mut self.content
    }
}

impl<C> TuiWidget for Tab<C>
where
    C: TabContent,
{
    fn handle_key_event(&mut self, kv: &KeyEvent) {
        if let Ok(key) = C::Key::try_from(kv) {
            self.content
                .handle_key_event(key, &mut self.tasks, &mut self.state)
        }
    }

    fn render(&mut self, f: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        self.content.render(f, area, &mut self.state);
    }

    fn sync(&mut self) {
        while let Some(f) = self.tasks.try_join_next() {
            f.unwrap()(self.content_mut())
        }
    }
}

impl<C> Default for Tab<C>
where
    C: TabContent + Default,
    C::State: Default,
{
    fn default() -> Self {
        Self {
            content: Default::default(),
            state: Default::default(),
            tasks: Default::default(),
        }
    }
}

/// Wrap a closure to [`CallBack`], used to wrap the return function of a future
///
/// e.g.
/// ``` rust,norun
/// let name = "test".to_owned();
/// let task = async {
///     println!("Start {}", name);
///     tokio::time::sleep(std::time::Duration::from_micros(10)).await;
///     wrapper(move |content: &mut Self| {
///         println!("Done {}", name);
///     })
/// };
/// task_set.spawn(task);
/// ```
pub fn wrapper<C>(f: impl FnOnce(&mut C) + 'static + Send) -> CallBack<C> {
    Box::new(f)
}

pub fn do_nothing<C>() -> CallBack<C> {
    wrapper(|_| ())
}
