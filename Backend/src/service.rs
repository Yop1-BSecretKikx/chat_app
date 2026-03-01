
use serde_json::{json, Value};
use sqlx::{types::chrono::Utc};

use crate::service;
pub enum CAUSE{
    CANNOT_PERFORME,
    BAD_REQUEST,
    WRONG_INFO,
    NONE
}
pub enum EMITTER{
    EMITE,
    NONE
}
pub enum STATUS{
    OK,
    FAIL
}
enum DATE{
    THIS_DATE,
}
impl STATUS{
    pub fn status(&self) -> String{
        match self{
            STATUS::OK => "OK".to_string(),
            STATUS::FAIL => "FAIL".to_string()
        }

    }
}
impl EMITTER{
    //to emit just do like its a "json" like builder u give a "name":"value" // so user app can catch it
    pub fn emite(&self,to_emit:Option<[&str;2]>) -> [String;2]{
        match self{
            EMITTER::EMITE => {
                if let Some(emit) = to_emit{
                    return [emit[0].to_string(),emit[1].to_string()];
                }else {
                    return ["".to_string(),"".to_string()];
                }
            }
            EMITTER::NONE => {
                return ["emit".to_string(), "NONE".to_string()];
            }
        }
    }
}
impl CAUSE{
    pub fn cause(&self) -> String{
        match self{
            CAUSE::CANNOT_PERFORME => "CANNOT PERFORME".to_string(),
            CAUSE::BAD_REQUEST => "BAD REQUEST".to_string(),
            CAUSE::WRONG_INFO => "WRONG INFO".to_string(),
            CAUSE::NONE => "NONE".to_string(),
        }
    }
}
impl DATE{
     pub fn date(&self) -> String{
        match self{
            DATE::THIS_DATE => {Utc::now().to_string()}
        }
    }
}
pub struct HTTP_TELL;
impl HTTP_TELL{
    pub fn TELL(emitter:[String;2],cause:String,status:String) -> Value
    {
        json!({
            emitter[0].clone():emitter[1],
            "STATUS":status.to_string(),
            "CAUSE":cause.to_string(),
            "DATE":service::DATE::THIS_DATE.date()
        })
    }
}