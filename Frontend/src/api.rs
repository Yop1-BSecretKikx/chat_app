use dioxus::logger::tracing;
pub use reqwest::Client;
use reqwest::Response;
pub use serde::Serialize;
use serde::Deserialize;
//api calls signup login send etc
pub struct user{
    token:String,
    username:String,
    date:String,
}
static ENTRY: &str = "https://zapo.up.railway.app/";

#[derive(Serialize)]
struct submited_info{
    username:String,
    password:String
}

/*
return HttpResponse::BadRequest().json(json!({
                                                "error":"NO",
                                                "status":"OK",
                                                "token":"no",
                                                "username":"no",
                                                "date":"no"
                                            }));
*/
#[derive(Deserialize,Debug)]
pub struct got_info {
    pub error:String,
    pub status:String,
    pub token:String,
    pub username:String,
    pub date:String
}
impl got_info{
    pub fn new()-> Self
    {
        Self { error: "".to_string(), status: "".to_string(), token: "".to_string(), username: "".to_string(), date: "".to_string() }
    }
    pub fn user_status(err:String,status:String,token:String,username:String,date:String)->Self{
        Self { error: err, status: status, token: token, username: username, date:date }
    }
}

pub async fn login_signin(mode:i32,_username:String,_password:String) -> got_info{
    let client = Client::new();
    let body = submited_info{
        username:_username,
        password:_password
    };

    match client.post(format!("{ENTRY}login")).json(&body).send().await{
        Ok(anser) => {
            match anser.json::<got_info>().await{
                Ok(got)=> {
                    
                    tracing::info!("{:?}",got);
                    if got.status == String::from("OK"){
                        return got_info::user_status(got.error, got.status, got.token, got.username, got.date);
                    }
                    else {
                        return got_info::user_status("error".to_string(), "FAIL".to_string(), "".to_string(), "".to_string(), "".to_string());
                    }
                },
                Err(e)=>{
                    println!("false bro");
                        return got_info::user_status("error".to_string(), "FAIL".to_string(), "".to_string(), "".to_string(), "".to_string());
                }
            }

        }
        Err(e)=> {
            return got_info::user_status("error".to_string(), "FAIL".to_string(), "".to_string(), "".to_string(), "".to_string());

        }
    }
}

#[derive(Deserialize,Debug)]
pub struct send_status {
    pub error:String
}
#[derive(Serialize)]
struct send_sub{
    pub message:String,
    pub token:String
}
pub async fn send_message(_message:String,_token:String) -> bool{
    let client = Client::new();

    let body = send_sub{
        message:_message,
        token:_token,
    };

    match client.post(format!("{ENTRY}send_data")).json(&body).send().await {
        Ok(anser) => {
            match anser.json::<send_status>().await {
                Ok(info) => {
                    if info.error == "no".to_string()
                    {
                        return true;
                    }
                    else {
                        return false;
                    }
                }
                Err(_)=>{return false;}
            };
        }
        Err(_)=>{false}
    }
    //false
}



#[derive(Serialize)]
struct pull_call{
    pub want_latest:bool
}

#[derive(Deserialize)]
struct MessageEntry {
    date: String,
    from_user: String,
    message: String,
}

#[derive(Deserialize)]
struct pull_recive{
#[serde(rename = "Data_base_message")]
    data_base_message: Vec<MessageEntry>,
}
pub async fn pull_latest(_latest:bool) -> Vec<(String, String, String)>{
    let client = Client::new();

    let body = pull_call{
        want_latest:_latest,
    };

    match client.post(format!("{ENTRY}pull")).json(&body).send().await{
        Ok(e) => {
            let t = e.text().await.unwrap();
            let parsed:pull_recive = serde_json::from_str(&t).unwrap();

            parsed.data_base_message
            .into_iter()
            .map(|m| (m.from_user, m.message, m.date))
            .collect()
        }
        Err(_)=> {vec![]}
    }

}