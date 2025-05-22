use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::{User, services::websocket::WebsocketService};
use crate::services::event_bus::EventBus;

pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
}

#[derive(Deserialize)]
struct MessageData {
    from: String,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    wss: WebsocketService,
    messages: Vec<MessageData>,
    _producer: Box<dyn Bridge<EventBus>>,
}
impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = serde_json::from_str(&s).unwrap();
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.into(),
                                avatar: format!(
                                    "https://avatars.dicebear.com/api/adventurer-neutral/{}.svg",
                                    u
                                )
                                .into(),
                            })
                            .collect();
                        return true;
                    }
                    MsgTypes::Message => {
                        let message_data: MessageData =
                            serde_json::from_str(&msg.data.unwrap()).unwrap();
                        self.messages.push(message_data);
                        return true;
                    }
                    _ => {
                        return false;
                    }
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    //log::debug!("got input: {:?}", input.value());
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
                        data_array: None,
                    };
                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                };
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);
        html! {
            <div class="flex h-screen w-screen font-sans">
                // Sidebar
                <div class="w-64 bg-white border-r border-gray-200 flex flex-col">
                    <div class="text-2xl font-semibold text-gray-700 p-4 border-b">{"👥 Users"}</div>
                    <div class="overflow-auto">
                        {
                            self.users.iter().map(|u| {
                                html! {
                                    <div class="flex items-center space-x-4 p-3 mx-2 my-2 rounded-lg hover:bg-gray-100 transition duration-200">
                                        <img class="w-10 h-10 rounded-full border" src={u.avatar.clone()} />
                                        <div>
                                            <p class="text-sm font-medium text-gray-800">{u.name.clone()}</p>
                                            <p class="text-xs text-gray-400">{"Hi there!"}</p>
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    </div>
                </div>

                // Chat Area
                <div class="flex flex-col flex-1">
                    <div class="h-14 flex items-center px-6 border-b text-xl font-semibold bg-gray-50">{"💬 Chat Room"}</div>
                    <div class="flex-1 overflow-y-auto px-6 py-4 space-y-4 bg-gray-50">
                        {
                            self.messages.iter().map(|m| {
                                let user = self.users.iter().find(|u| u.name == m.from);
                                if let Some(user) = user {
                                    html! {
                                        <div class="flex items-start space-x-3">
                                            <img class="w-8 h-8 rounded-full border" src={user.avatar.clone()} />
                                            <div>
                                                <p class="text-sm font-medium text-gray-800">{m.from.clone()}</p>
                                                {
                                                    if m.message.ends_with(".gif") {
                                                        html! { <img src={m.message.clone()} class="mt-2 max-w-xs rounded-lg shadow-sm"/> }
                                                    } else {
                                                        html! { <p class="mt-1 text-sm bg-white p-3 rounded-lg shadow-sm text-gray-800">{m.message.clone()}</p> }
                                                    }
                                                }
                                            </div>
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }
                            }).collect::<Html>()
                        }
                    </div>

                    // Chat Input
                    <div class="h-16 flex items-center px-4 bg-white border-t">
                        <div class="flex items-center w-full space-x-3">
                            <input
                                ref={self.chat_input.clone()}
                                type="text"
                                placeholder="Type a message..."
                                class="flex-grow py-2 px-4 bg-gray-100 rounded-full text-sm focus:outline-none focus:ring-2 focus:ring-blue-400"
                            />
                            <button
                                onclick={submit}
                                class="flex items-center justify-center w-10 h-10 bg-blue-600 hover:bg-blue-700 text-white rounded-full transition duration-200 shadow"
                            >
                                <svg class="w-5 h-5" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
                                </svg>
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}