
use crate::{api::got_info, chat::chat_app, entry::Entry};
mod entry;
use dioxus::{html::div, prelude::*};
mod api;
mod chat;
const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");

fn main() {
   dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut user_struct = use_signal(|| got_info::new());

    let mut visible = use_signal(|| true);
    if visible() {
        rsx! {
        div {
            id:"login_entry",
            document::Link { rel: "icon", href: FAVICON }
            document::Link { rel: "stylesheet", href: MAIN_CSS }
            Entry {
                on_succes:move |got:got_info|{
                   if got.status == "OK".to_string(){
                    user_struct.set(got);
                    visible.set(false);
                   }
                },
            }
        }
    }
    }else {
        let user = vec![
            user_struct.read().username.clone(),
            user_struct.read().token.clone(),
            user_struct.read().date.get(0..=10).unwrap().to_string(),
        ];
        chat_app(user)
    }
}