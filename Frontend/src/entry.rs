use std::os::unix::thread;

use dioxus::{core::Runtime, html::div, prelude::*};
use crate::api::{got_info, login_signin};

/*
    -
    -
    -
*/
#[derive(Props, Clone, PartialEq)]
pub struct EntryProps{
    on_succes:EventHandler<got_info>,
}
#[component]
pub fn Entry(props:EntryProps) -> Element{

    let mut username = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());

    let mut button_state = use_signal(|| String::from("Login"));
    
    rsx! {
        div
        {
            id:"Login_Signin_space",
            input {
            id:"username_textfield-id",
            r#type:"text",
            placeholder:"username",
            value:{username},
            oninput:move|e|username.set(e.value())
        }
        input {
            id:"password_textfield-id",
            r#type:"text",
            placeholder:"password",
            value:{password},
            oninput:move|e|password.set(e.value())
        }
        button {
            id:"button_login-id",
            onclick:move |_|{
                dioxus::prelude::spawn(async move {
                    let latest_button_state = button_state.to_string();
                    button_state.set("Wait".to_string());

                    let got = login_signin(1, username.to_string(), password.to_string()).await;
                    props.on_succes.call(got);
                    button_state.set(latest_button_state);
                });
            },
            "{button_state}",
        }
        }
    }
}
