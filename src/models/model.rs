use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub username: String,
    pub password: String,
    pub email: String,
    pub phonenumber: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlogPost {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub title: String,
    pub content: String,
    pub author_id: ObjectId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePostRequest {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdatePostRequest {
    pub title: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug,Serialize,Deserialize)]
pub struct RegisterRequest{
    pub username :String,
    pub email : String,
    pub password:String,
    pub phonenumber:String
}
