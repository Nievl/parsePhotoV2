use std::cell::RefCell;
use std::rc::Rc;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct Notification {
    pub id: usize,
    pub message: String,
    pub notification_type: String, // Например: "success", "error", "info"
}

#[derive(Clone)]
pub struct NotificationContext {
    notifications: Rc<RefCell<Vec<Notification>>>,
    add_notification: Callback<(String, String)>, // message, notification_type
    remove_notification: Callback<usize>,         // id
}

impl NotificationContext {
    pub fn new() -> Self {
        let notifications = Rc::new(RefCell::new(vec![]));
        let notifications_clone = notifications.clone();

        let add_notification =
            Callback::from(move |(message, notification_type): (String, String)| {
                let mut notifications = notifications_clone.borrow_mut();
                let id = notifications.len() + 1; // Простое создание ID
                notifications.push(Notification {
                    id,
                    message,
                    notification_type,
                });
            });

        let notifications_clone = notifications.clone();
        let remove_notification = Callback::from(move |id: usize| {
            let mut notifications = notifications_clone.borrow_mut();
            notifications.retain(|n| n.id != id);
        });

        Self {
            notifications,
            add_notification,
            remove_notification,
        }
    }

    pub fn use_context() -> NotificationContext {
        use_context::<NotificationContext>().expect("No NotificationContext found!")
    }
}

impl PartialEq for NotificationContext {
    fn eq(&self, _other: &Self) -> bool {
        false // Обновляем при каждом изменении
    }
}

#[function_component]
pub fn NotificationContainer() -> Html {
    let context = NotificationContext::use_context();

    let notifications = {
        let notifications = context.notifications.borrow();
        notifications.clone()
    };

    html! {
        <div class="notification-container">
            { for notifications.iter().map(|notification| {
                let remove_callback = {
                    let remove_notification = context.remove_notification.clone();
                    let id = notification.id;
                    Callback::from(move |_| {
                        remove_notification.emit(id);
                    })
                };

                html! {
                    <div class={classes!("notification", &notification.notification_type)}>
                        <p>{ &notification.message }</p>
                        <button onclick={remove_callback}>{ "Close" }</button>
                    </div>
                }
            })}
        </div>
    }
}
