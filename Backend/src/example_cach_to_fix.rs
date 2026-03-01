use actix_web::{App, Error, HttpResponse, HttpServer, Responder, cookie::time::error, http::header::DATE, rt::Runtime, web::{self, post}};
use serde::de::IntoDeserializer;
use tokio::runtime;
use std::{env, sync::Mutex, time::Duration};
use actix_web::get;
use sqlx::{Database, Pool, database, mysql::MySqlPoolOptions, query, types::chrono::Utc};
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
use std::sync::Arc;
use crate::service::{CAUSE, EMITTER, HTTP_TELL, STATUS};
mod service;
/*
code working next step async thread check memory usage if high send data of struct block_mess

*/


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
fn mb_size(to_analyse:&Vec<(String,String,String)>) -> f64 {
        let mut bytes = 0;
        bytes += to_analyse.capacity() * std::mem::size_of::<(String, String, String)>();
        for (a, b, c) in to_analyse.iter() {
            bytes += a.capacity();
            bytes += b.capacity();
            bytes += c.capacity();
        }
        bytes as f64 / (1024.0 * 1024.0)
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

struct block_mess{
    block:Arc<Mutex<Vec<(String,String,String)>>>
}
impl block_mess{
    fn new() -> Self{
        Self { block:Arc::new(Mutex::new(Vec::from(vec![(".".to_string(),".".to_string(),".".to_string())])))}
    }
    async fn init_latest(&self,pool:&MySqlPool) -> Result<(),sqlx::Error>{
        let start = Instant::now();
        let query = sqlx::query!("SELECT * FROM Global ORDER BY date DESC LIMIT ?",100).fetch_all(pool).await?;
        for row in query{
            self.block.lock().unwrap().push((row.from_user,row.latest_message,row.date.to_string()));
        }
        println!("latest message init in {:?}",start.elapsed());
        Ok(())
    }
    async fn update_db(&self,pool:&MySqlPool){
        let block_cpy = self.block.clone();
        let pool = pool.clone();
        tokio::spawn(async move {
            let limit_safe = 0.0008;
            loop {
                {
                    let latest:Vec<(String,String,String)> = {
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        let mut block = block_cpy.lock().unwrap();
                        if mb_size(&block) < limit_safe {
                            println!("Using {}mb",mb_size(&block));
                            continue;
                        }else {
                            println!("Using {}mb",mb_size(&block));
                        }
                        println!("[info] reached a health limit cleaning");

                        let mut drain_count = 0;
                        while mb_size(&block) >= limit_safe && drain_count < block.len()
                        {
                            drain_count += 1;
                        }
                        block.drain(..4).collect()
                    };
                    println!("[+] Updating db");
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
        });
        //send on hight usage
    }
}

#[derive(Deserialize)]
struct PullSpesification{
    want_latest:bool,
}

//Pull data to fix + security token checker !
#[post("/pull")]
async fn _pull(data:web::Json<PullSpesification>,pool:web::Data<MySqlPool>,mess_block:web::Data<block_mess>) -> impl Responder
{
    let start = Instant::now();
    let utc = Utc::now().to_string();
    

    //let query = sqlx::query!("SELECT * FROM Global ORDER BY date DESC LIMIT ?",100).fetch_all(pool.get_ref()).await;
    let mut message_contener = Vec::new();

    for data_ in mess_block.block.lock().unwrap().iter(){
        message_contener.push(json!({
            "from_user":*data_.0,
            "message":*data_.1,
            "date":*data_.2
        }));
        if data.want_latest == true{
            println!("Pulled data in {:?}",start.elapsed());
            break;
        }
    }

    return HttpResponse::Ok().json(json!({
        "Data_base_message":message_contener
    }));
}
//
#[derive(Deserialize)]
struct from_sender{
    message:String,
    token:String
} 
#[post("/send_data")]
async fn _send(data:web::Json<from_sender>,mess_block:web::Data<block_mess>,pool:web::Data<MySqlPool>) -> impl Responder{
    let start = Instant::now(); 
    if data.message.is_empty(){
        return HttpResponse::BadRequest().json(json!({
            "error":"fail"
        }));
    }
    let query = sqlx::query!("SELECT username FROM users WHERE session_token = ?",data.token).fetch_one(pool.get_ref()).await;
    match query {
        Ok(record) => {
            let utc = Utc::now().to_string();
            mess_block.block.lock().unwrap().insert(0,(record.username.clone(),data.message.clone(),utc.clone()));
            println!("message insert in {:?}",start.elapsed());
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
                                                    "error": "NO",                                                
                                                    "status": "OK",
                                                    "token": info.session_token.to_string(),                                               
                                                    "username": data.username,                                               
                                                    "date":utc                 
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

    println!("Server running at 0.0.0.0:{} v0.0.2", port);
    let database_url = std::env::var("DATABASE_URL").unwrap();

    //check if we got a succes connection
    //let sql_pool = sqlx::MySqlPool::connect(&database_url).await.map_err(|e| {
     //   std::io::Error::new(std::io::ErrorKind::Other, format!("[info] DB error: {} | FAIL", e))
    //})?;
    
    let sql_pool = MySqlPoolOptions::new().max_connections(100).min_connections(3).connect(&database_url).await.unwrap();
        
        
        let data_mess= web::Data::new(block_mess::new());
        data_mess.init_latest(&sql_pool).await;
        data_mess.update_db(&sql_pool).await;
        let data_mess_clone = data_mess.clone();
        HttpServer::new(move || App::new()
        .app_data(web::Data::new(sql_pool.clone()))
        .app_data(data_mess_clone.clone())
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

//db struct
/*
    -Global
        -from_user
        -latest_message
    
    -User
        -username
        -password
        -creation_date
        -session_token


    curl -X POST http://localhost:3000/login `
  -H "Content-Type: application/json" `
  -d '{\"username\":\"test\",\"password\":\"test\"}'
*/


/*

server {
    listen 443 ssl;
    server_name 196.112.190.70;  # ou ton domaine si tu en as un

    ssl_certificate     cert.pem;   # certificat auto-signé
    ssl_certificate_key key.pem;

    location / {
        proxy_pass http://127.0.0.1:9933;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}

Pour créer un certificat auto-signé :

openssl req -x509 -nodes -days 365 -newkey rsa:2048 -keyout key.pem -out cert.pem
*/
/*
CREATE DATABASE IF NOT EXISTS moox_db;
USE moox_db;

-- Table des utilisateurs
CREATE TABLE IF NOT EXISTS users (
    username VARCHAR(50) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    creation_date DATETIME NOT NULL,
    session_token VARCHAR(255) NOT NULL
);

-- Table des messages globaux
CREATE TABLE IF NOT EXISTS Global (
    from_user VARCHAR(50) NOT NULL,
    latest_message TEXT NOT NULL,
    date DATETIME NOT NULL
);

*/