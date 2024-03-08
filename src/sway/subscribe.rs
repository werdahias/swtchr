use eyre::WrapErr;
use std::sync::{mpsc, Arc, RwLock};
use std::thread;
use swayipc::{self, Connection, Event, EventType, WindowChange};

use super::queue::WindowQueue;

fn filter_event(
    event_result: swayipc::Fallible<Event>,
    urgent_first: bool,
) -> eyre::Result<Option<WindowEvent>> {
    let event = event_result.wrap_err("failed reading Sway event result")?;

    let window_event = match event {
        Event::Window(window_event) => window_event,
        _ => return Ok(None),
    };

    match window_event.change {
        WindowChange::New | WindowChange::Focus | WindowChange::Urgent => {
            if window_event.change == WindowChange::Urgent
                && (!urgent_first || !window_event.container.urgent)
            {
                // One of two things has happened:
                // 1. Urgent-first window ordering is turned off.
                // 2. The window urgency changed, but from being urgent to being not-urgent. This
                //    shouldn't affect the order in the window switcher.
                return Ok(None);
            }

            match Window::from_node(window_event.container) {
                Some(sway_window) => Ok(Some(WindowEvent::Focus(sway_window))),
                None => Ok(None),
            }
        }
        WindowChange::Close => Ok(Some(WindowEvent::Close(window_event.container.id))),
        _ => Ok(None),
    }
}

pub struct WindowSubscription {
    // Send a command to the actor thread to send the current sorted window list.
    command: mpsc::Sender<()>,

    // Receive the current sorted window list.
    response: mpsc::Receiver<eyre::Result<Vec<Window>>>,
}

impl WindowSubscription {
    pub fn subscribe(urgent_first: bool) -> eyre::Result<WindowSubscription> {
        let (command_sender, command_reciever) = mpsc::channel();
        let (response_sender, response_reciever) = mpsc::channel();

        let connection = Connection::new().wrap_err("failed acquiring a Sway IPC connection")?;
        let subscription = connection
            .subscribe([EventType::Window])
            .wrap_err("failed opening a Sway window event subscription")?;

        let pushing_window_queue = Arc::new(RwLock::new(WindowQueue::new()));
        let listing_window_queue = Arc::clone(&pushing_window_queue);

        let err_response_sender = Arc::new(response_sender);
        let ok_response_sender = Arc::clone(&err_response_sender);

        // Subscribe to events from the Sway IPC API and update the window priority queue.
        thread::spawn(move || {
            for event_result in subscription {
                if let Some(result) = filter_event(event_result, urgent_first).transpose() {
                    match result {
                        Ok(event) => {
                            pushing_window_queue
                                .write()
                                .expect("window queue lock is poisoned")
                                .push_event(event);
                        }
                        Err(err) => err_response_sender
                            .send(Err(err))
                            .expect("channel closed unexpectedly"),
                    }
                } else {
                    continue;
                }
            }
        });

        // Wait for a command to get the current sorted list of windows from the window priority
        // queue.
        thread::spawn(move || loop {
            command_reciever
                .recv()
                .expect("channel closed unexpectedly");

            ok_response_sender
                .send(Ok(listing_window_queue
                    .read()
                    .expect("channel closed unexpectedly")
                    .sorted_windows()))
                .expect("");
        });

        Ok(Self {
            command: command_sender,
            response: response_reciever,
        })
    }

    pub fn get_window_list(&self) -> eyre::Result<Vec<Window>> {
        self.command.send(()).expect("channel closed unexpectedly");
        self.response.recv().expect("channel closed unexpectedly")
    }
}

pub type SwayNodeId = i64;

pub enum WindowEvent {
    // A window was focused, created, or marked urgent.
    Focus(Window),

    // A window was closed.
    Close(SwayNodeId),
}

#[derive(Debug, Clone)]
pub struct Window {
    pub id: SwayNodeId,
    pub window_title: String,
    pub app_id: String,
}

impl Window {
    // Returns `None` if the node is not a view.
    fn from_node(node: swayipc::Node) -> Option<Self> {
        if let (Some(name), Some(app_id)) = (node.name, node.app_id) {
            Some(Self {
                id: node.id,
                window_title: name,
                app_id,
            })
        } else {
            None
        }
    }
}
