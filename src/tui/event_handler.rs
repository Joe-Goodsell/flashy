use color_eyre::eyre;
use crossterm::event::KeyEvent;
use tokio::{sync::mpsc, task::JoinHandle};
use futures::{FutureExt, StreamExt};


#[derive(Debug)]
pub enum Event {
    Error,
    Tick,
    Key(KeyEvent),
}

#[derive(Debug)]
pub struct EventHandler {
    _tx: mpsc::UnboundedSender<Event>,
    rx: mpsc::UnboundedReceiver<Event>,
    task: Option<JoinHandle<()>>,
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new(None)
    }
}

impl EventHandler {
    pub fn new(tick_rate: Option<std::time::Duration>) -> Self {
        let tick_rate = match tick_rate {
            Some(tr) => tr,
            _ => std::time::Duration::from_millis(250)
        };

        let (tx, rx) = mpsc::unbounded_channel();
        let _tx = tx.clone();

        let task = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut interval = tokio::time::interval(tick_rate);

            loop {
                let delay = interval.tick();
                let crossterm_event = reader.next().fuse();
                tokio::select! {

                    maybe_event = crossterm_event => {
                        match maybe_event {
                            Some(Ok(evt)) => {
                                match evt {
                                    crossterm::event::Event::Key(key) => {
                                        if key.kind == crossterm::event::KeyEventKind::Press {
                                            tx.send(Event::Key(key)).unwrap();
                                        }
                                    },
                                    _ => {},
                                }
                            }
                            Some(Err(_)) => {
                                tx.send(Event::Error).unwrap();
                            },
                            None => {},
                        }
                    },

                    _ = delay => {
                        tx.send(Event::Tick).unwrap();
                    },

                }
            }
        });    

        Self { _tx, rx, task: Some(task) }
    }

    pub async fn next(&mut self) -> eyre::Result<Event> {
        self.rx.recv().await.ok_or(color_eyre::eyre::eyre!("Unable to get event"))
    }
}