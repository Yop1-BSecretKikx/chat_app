use std::time::Duration;

use dioxus::{core::{Element, IntoAttributeValue}, html::{elements, mover}, logger::tracing, prelude::*};
use async_std::{io::empty, task::sleep};
use crate::api::{pull_latest, send_message};

#[component]
fn messages(author:String,message:String,date:String) -> Element{
    rsx!{
        div {
            id:"message-box",
            p {
                "{date}",
                "{author}"
            }
            span {"{message}"}
        }
    }
}
pub fn chat_app(user:Vec<String>)-> Element{
    let mut show_profil = use_signal(|| false);
    let token_preview = format!("{}...{}",&user[1][0..=3],&user[1][user.len() - 2..=user.len()]);
    let mut message = use_signal(|| String::new());
    let mut state = use_signal(|| String::from("post"));

    let mut messages_users: Signal<Vec<(String,String,String)>> = use_signal(|| vec![]);
    use_effect(move ||{
        dioxus::prelude::spawn(async move {
            messages_users.set(pull_latest(false).await);
        });
    });
    use_effect(move || {
    spawn(async move {
        let mut latest_block = pull_latest(false).await[0].clone();

        loop {
            let latest = pull_latest(true).await[0].clone();
            sleep(Duration::from_millis(100)).await;

            if latest_block != latest {
                
                if !pull_latest(false).await.is_empty(){
                    latest_block = pull_latest(false).await[0].clone();

                    messages_users.write().insert(0, (latest.0,latest.1,latest.2));
                }
                
            } else {
                println!("nothing");
            }
        }
    });
});
    rsx!{
        div {
            id:"chat_app_id",
            div {
                style: "display: flex; align-items: center;",
                id:"top-div-chat",
                span {"Connected as {user[0]}"},button {id:"status", ""},
                button {
                    onclick:move|_|{
                        show_profil.set(!show_profil());
                    },
                    //on peut executer le if ici ?
                    id:"profils", "",
                }
                if show_profil(){
                    div {
                        id:"profils-box",
                        span {"username : {user[0]}"},
                        span {"token : {token_preview}"},
                        span {"created : {user[2]}"}
                    }
                }
            }
            div {
                id: "chat-scrollable",
                    for (username,message,date) in messages_users.read().iter(){
                        div {
                            id: "Mess-box",
                            div {
                            class: "mess-header",
                            span { class: "mess-username", "{username}" }
                            span { class: "mess-date", "{date}" }
                        }
                        span { class: "mess-content", "{message}" }
                }
                }
            }
            div {
                input {
                    id:"Text-input",
                    placeholder:"say somthing don't be shay bro !",
                    value:{message},
                    oninput:move |e|{
                        message.set(e.value());
                    },
                }
                button {
                    id:"to_send",
                    onclick:move |_|
                    {
                        state.set("wait".to_string());
                        let token = user[1].clone();
                        let token_preview = token_preview.clone();
                        dioxus::prelude::spawn(async move {
                            
                            if send_message(message(), token.clone()).await == true{
                                tracing::info!("Send !");
                                message.set("".to_string());
                                
                            }
                            else {
                                tracing::info!("Fail for token : {} message : {}",token_preview,message);
                            }
                            state.set("post".to_string());
                        });
                    },
                    "{state()}",
                }
            }
        }
        
    }
} 