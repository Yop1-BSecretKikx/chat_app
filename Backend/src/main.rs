use actix_web::{App, Error, HttpResponse, HttpServer, Responder, cookie::time::error, http::header::DATE, rt::Runtime, web::{self, post}};
use serde::de::IntoDeserializer;
use tokio::runtime;
use std::{env, time::Duration};
use actix_web::get;
use sqlx::{Database, Pool, mysql::MySqlPoolOptions, query, types::chrono::Utc};
use dotenvy::dotenv;
use sqlx::MySqlPool;
use serde_json::{json,value};
use serde::Deserialize;
use actix_web::post;
use argon2::{
    Argon2, password_hash::{PasswordHasher, PasswordVerifier,PasswordHash,SaltString, rand_core::{OsRng, RngCore}}  
};
use base64::{engine::general_purpose, Engine as _};
use std::time::Instant;

use crate::service::{CAUSE, EMITTER, HTTP_TELL, STATUS};
mod service;
fn generate_session_token() -> String{
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);

    let l = general_purpose::URL_SAFE_NO_PAD.encode(bytes);
    return  l;
}
fn password_check(user_post_password:&String,data_base_password_hash:&String) -> bool{
    let hash_parse = PasswordHash::new(data_base_password_hash);
    let mut checked = false;
    if let Ok(parsed) = hash_parse{
        checked =  Argon2::default().verify_password(user_post_password.as_bytes(), &parsed).is_ok();
    }
    return checked;
}

#[get("/")]
async fn root() -> impl Responder {

    HttpResponse::Ok().body("Root /")
}

#[derive(Deserialize)]
struct CreateUserRequest {
    username: String,
    password: String,
}
#[post("/signin")]
async fn _signin(data: web::Json<CreateUserRequest>,pool: web::Data<MySqlPool>) -> impl Responder{
    //generate utc time

    //generate a salt for password hashing

    if data.username.is_empty() || data.password.is_empty(){
        return HttpResponse::BadRequest().json(json!({
            "error":"username or password empty"
        }));
    }

    if data.username.len() > 50 || data.password.len() > 30{
        return HttpResponse::BadRequest().json(json!({
            "error":"to long username or password"
        }))
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hashed_password = argon2.hash_password(data.password.as_bytes(), &salt).unwrap().to_string();

    match sqlx::query!("INSERT INTO users (username, password, creation_date,session_token) VALUES (?, ?, ?,?)",data.username,hashed_password,&Utc::now(),generate_session_token()).execute(pool.get_ref()).await {
        Ok(_) => {
            println!("[info] create_acc request succes | OK");
            return HttpResponse::Created().json(
                HTTP_TELL::TELL(
                    //do we return the session token acc or no ?
                    EMITTER::EMITE.emite(None),
                    CAUSE::NONE.cause(),
                    STATUS::OK.status()
                )
            );
        },
        Err(error) => {
            let mut message = String::new();
            if error.to_string().contains("Duplicate"){
                message = String::from("username already taken");
            }else {
                message = String::from("cannot create account");
            }
            return HttpResponse::Conflict().json(json!({
            "error":message
        }))
        }
    }
}

#[derive(Deserialize)]
struct PullSpesification{
    want_latest:bool,
}
//Pull data to fix + security token checker !
#[post("/pull")]
async fn _pull(data:web::Json<PullSpesification>,pool:web::Data<MySqlPool>) -> impl Responder
{
    let utc = Utc::now().to_string();
    //
    println!("[info : {}] recived a pool call with latest?({})| OK",&utc,data.want_latest);
    let query = sqlx::query!("SELECT * FROM Global ORDER BY date DESC LIMIT ?",100).fetch_all(pool.get_ref()).await;
    let mut message_contener = Vec::new();
    match query {
        Ok(pooled_data) => {
            for mess in pooled_data{    
                message_contener.push(json!({
                    "from_user":mess.from_user,
                    "message":mess.latest_message,
                    "date":mess.date
                }));
                if data.want_latest == true{
                    break;
                }
            }
            println!("[info : {}] pool call fetched | OK",&utc);
            return HttpResponse::Ok().json(json!({
                "Data_base_message":message_contener
            }));
        },
        Err(error) => {
            println!("[info : {}] pool call fetched | NO",&utc);
            return HttpResponse::InternalServerError().json(json!({
                "task":"Pool message",
                "cause":error.to_string()
            }));
    }
}

}

#[derive(Deserialize)]
struct from_sender{
    message:String,
    token:String
} 
#[post("/send_data")]
async fn _send(data:web::Json<from_sender>,pool:web::Data<MySqlPool>) -> impl Responder{
    if data.message.is_empty(){
        return HttpResponse::BadRequest().json(json!({
            "error":"fail"
        }));
    }
    let query = sqlx::query!("SELECT username FROM users WHERE session_token = ?",data.token).fetch_one(pool.get_ref()).await;
    match query {
        Ok(record) => {
            let utc = Utc::now().to_string();
            match sqlx::query!("INSERT INTO Global (from_user,latest_message,date) VALUES (?,?,?)",record.username,data.message,utc).execute(pool.get_ref()).await{
                Ok(_)=>{
                    return HttpResponse::Ok().json(json!({
                        "error":"no"
                    }));
                }
                Err(_)=>{
                     return HttpResponse::Ok().json(json!({
                        "error":"fail"
                    }));
                }
            }

        }
        Err(_)=>{
             return HttpResponse::Ok().json(json!({
                    "error":"fail"
            }));
        }
    }
    //unles
    return HttpResponse::Unauthorized().json(json!({"error":"fail"}));


}
#[derive(Deserialize)]
struct login{
    username:String,
    password:String
}
#[post("/login")]
async fn _login(data:web::Json<login>,pool:web::Data<MySqlPool>) -> impl Responder{
    let start = Instant::now();
    //query to find username if existe if yes get the hashed password then parse it. then verify password in the post body whith the parsed password. 
    if data.username.is_empty() || data.password.is_empty(){
        println!("Login Done In {:?}",start.elapsed());
        return HttpResponse::BadRequest().json(json!({
            "error":"username or password empty"
        }));
    }

    if data.username.len() > 50 || data.password.len() > 30{
        println!("Login Done In {:?}",start.elapsed());
        return HttpResponse::BadRequest().json(json!({
            "error":"incorrect username or password"
        }))
    }
    let utc = Utc::now().to_string();
    match sqlx::query_scalar!("SELECT EXISTS(SELECT 1 FROM users WHERE username = ?)",data.username).fetch_one(pool.get_ref()).await {
        Ok(query_response) => {
            match query_response {
                1 => {
                    //here is assume the username if found in the db we are supposed to catch the user hashed password and compare it with the password passed in the body request
                    //catch the username passed hashed passsword
                    match sqlx::query!("SELECT password FROM users WHERE username = ?",data.username).fetch_optional(pool.get_ref()).await{
                        Ok(hashed_password) => {
                            //getting the db hash password from that user now we compare it with the password user put in the post request to see if it match
                            if let Some(pass) = hashed_password{
                                //&data.password we will ensure that it will be hashed from the user app side and un-hashed here in the future, so we make sure that if the password is correct
                                //it will not travel nuded
                                if let true = password_check(&data.password, &pass.password){ 
                                    let query_get_token = sqlx::query!("SELECT session_token,username,creation_date FROM users WHERE username = ?",data.username).fetch_one(pool.get_ref()).await;
                                    match query_get_token {
                                        Ok(info) => {
                                            println!("Login Done In {:?} at {}",start.elapsed(),utc);
                                            return HttpResponse::Ok().json(json!({
                                                "error":"NO",
                                                "status":"OK",
                                                "token":info.session_token.to_string(),
                                                "username":data.username,
                                                "date":info.creation_date
                                            }));
                                        },
                                        //internal error
                                        Err(error) => {
                                            println!("Login Done In {:?}",start.elapsed());
                                            return HttpResponse::BadRequest().json(json!({
                                                "error":"NO",
                                                "status":"FAIL",
                                                "token":"no",
                                                "username":"no",
                                                "date":"no"
                                            }));
                                        }
                                    };
                                    //here u return the session token so in the userapp we catch it so once a logged user send a messsage he will send the message as that user session and the session is
                                    // appropriated to the spesifique user so once he send message he will send it as that token and that token is sort of connected to the user and password !
                                }else {
                                    println!("Login Done In {:?}",start.elapsed());
                                    return HttpResponse::BadRequest().json(json!({
                                                "error":"NO",
                                                "status":"FAIL",
                                                "token":"no",
                                                "username":"no",
                                                "date":"no"
                                            }));
                                }
                            }
                        },
                        Err(error) => {
                            println!("Login Done In {:?}",start.elapsed());
                            return HttpResponse::BadRequest().json(json!({
                                "error":"NO",
                                "status":"FAIL",
                                "token":"no",
                                "username":"no",
                                "date":"no"
                            }));
                        }
                    };
                },
                0 => {
                    println!("Login Done In {:?}",start.elapsed());
                    return HttpResponse::BadRequest().json(json!({
                        "error":"NO",
                        "status":"FAIL",
                        "token":"no",
                        "username":"no",
                        "date":"no"
                    }));  
                }
                _ => {}
            }
        },
        Err(error) => {
            println!("Login Done In {:?}",start.elapsed());
            return HttpResponse::BadRequest().json(json!({
                "error":"NO",
                "status":"FAIL",
                "token":"no",
                "username":"no",
                "date":"no"
            }));
            },
        };
    HttpResponse::Ok().body("")
}

//#[actix_web::main]
#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() -> std::io::Result<()> {
    let port: u16 = env::var("3000")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("err_port");
    dotenv().ok();

    println!("Server running at 0.0.0.0:{}", port);
    let database_url = std::env::var("DATABASE_URL").unwrap();

    //check if we got a succes connection
    //let sql_pool = sqlx::MySqlPool::connect(&database_url).await.map_err(|e| {
     //   std::io::Error::new(std::io::ErrorKind::Other, format!("[info] DB error: {} | FAIL", e))
    //})?;
    let sql_pool = MySqlPoolOptions::new().max_connections(100).min_connections(3).connect(&database_url).await.unwrap();
        println!("[info] DB connected | OK");
        HttpServer::new(move || App::new()
        .app_data(web::Data::new(sql_pool.clone()))
        .service(root)
        .service(_signin)
        .service(_pull)
        .service(_send)
        .service(_login)
        )
        .workers(num_cpus::get())
        .bind(("0.0.0.0", port))?
        .run()
        .await
    }
