use axum::{
    extract::{Path, State}, http::StatusCode, routing::{delete, get, post, put}, Json, Router
};
use bcrypt::{hash, verify};
use mongodb::{bson::doc, options::IndexOptions, Collection, Database, IndexModel};
use futures::TryStreamExt;

use crate::{
    auth::{self, create_jwt, AuthUser},
    error::AppError,
    models::model::{
        BlogPost, CreatePostRequest, LoginRequest, RegisterRequest, UpdatePostRequest, User,
    },
};

pub fn create_router(db: Database) -> Router {
    let db_clone = db.clone();
    // Create the unique index for email
    tokio::spawn(async move {
        if let Err(e) = create_user_index(&db_clone).await {
            tracing::error!("Failed to create user index: {:?}", e);
        }
    });
    Router::new()
    .route("/register", post(register))
    .route("/login", post(login))
    .route("/posts", post(create_post).get(get_posts))
    .route("/posts/:id", get(get_post).put(update_post).delete(delete_post))
    .with_state(db)
}

pub async fn create_user_index(db: &Database) -> Result<(), AppError> {
    let users_coll: Collection<User> = db.collection("users");
    let options = IndexOptions::builder().unique(true).build();
    let model = IndexModel::builder()
        .keys(doc! {"email":1})
        .options(options)
        .build();

    users_coll.create_index(model, None).await?;
    Ok(())
}

async fn register(
    State(db): State<Database>,
    Json(register_req): Json<RegisterRequest>,
) -> Result<Json<User>, AppError> {
    let users_coll: Collection<User> = db.collection("users");

    if users_coll
        .find_one(doc! {"email":&register_req.email}, None)
        .await?
        .is_some()
    {
        return Err(AppError::BadRequest("Email already in use".to_string()));
    }

    let hashed_password = hash(&register_req.password, 10)
        .map_err(|_| AppError::BadRequest("Failed to hash password".to_string()))?;

    let new_user = User {
        id: None,
        username: register_req.username,
        password: hashed_password,
        email: register_req.email,
        phonenumber: register_req.phonenumber,
    };
    users_coll.insert_one(&new_user, None).await?;
    Ok(Json(new_user))
}
async fn login(
    State(db): State<Database>,
    Json(login): Json<LoginRequest>,
) -> Result<Json<String>, AppError> {
    let users_coll: Collection<User> = db.collection("users");
    let user = users_coll
        .find_one(doc! { "email": &login.email }, None)
        .await?
        .ok_or_else(|| AppError::Auth("User not found".to_string()))?;

    if !verify(&login.password, &user.password)
        .map_err(|_| AppError::Auth("Invalid password".to_string()))?
    {
        return Err(AppError::Auth("Invalid password".to_string()));
    }

    let token = create_jwt(&user.id.unwrap().to_hex())?;
    Ok(Json(token))
}

async fn create_post(
    State(db): State<Database>,
    auth_user: AuthUser,
    Json(post): Json<CreatePostRequest>,
) -> Result<Json<BlogPost>, AppError> {
    let posts_coll: Collection<BlogPost> = db.collection("posts");
    let new_post = BlogPost {
        id: None,
        title: post.title,
        content: post.content,
        author_id: auth_user.0.parse().map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?,
    };
    posts_coll.insert_one(&new_post, None).await?;
    Ok(Json(new_post))
}

async fn get_posts(
    State(db): State<Database>,
) -> Result<Json<Vec<BlogPost>>, AppError> {
    let posts_coll: Collection<BlogPost> = db.collection("posts");
    let mut cursor = posts_coll.find(None, None).await?;
    let mut posts = Vec::new();
    while let Some(post) = cursor.try_next().await? {
        posts.push(post);
    }
    Ok(Json(posts))
}
async fn get_post(
    State(db): State<Database>,
    Path(id): Path<String>,
) -> Result<Json<BlogPost>, AppError> {
    let posts_coll: Collection<BlogPost> = db.collection("posts");
    let post = posts_coll
        .find_one(doc! { "_id": id.parse::<mongodb::bson::oid::ObjectId>().map_err(|_| AppError::BadRequest("Invalid ID".to_string()))? }, None)
        .await?
        .ok_or_else(|| AppError::NotFound("Post not found".to_string()))?;
    Ok(Json(post))
}

async fn update_post(
    State(db): State<Database>,
    auth_user: AuthUser,
    Path(id): Path<String>,
    Json(update): Json<UpdatePostRequest>,
)-> Result<Json<BlogPost>, AppError>{
    let posts_coll: Collection<BlogPost> = db.collection("posts");
    let object_id = id.parse::<mongodb::bson::oid::ObjectId>().map_err(|_| AppError::BadRequest("Invalid ID".to_string()))?;
    let post = posts_coll
        .find_one(doc! { "_id": &object_id }, None)
        .await?
        .ok_or_else(|| AppError::NotFound("Post not found".to_string()))?;

    if post.author_id.to_hex() != auth_user.0 {
        return Err(AppError::Auth("Not authorized to update this post".to_string()));
    }

    let update_doc = doc! {
        "$set": {
            "title": update.title.unwrap_or(post.title),
            "content": update.content.unwrap_or(post.content),
        }
    };

    posts_coll.update_one(doc! { "_id": &object_id }, update_doc, None).await?;
    
    let updated_post = posts_coll
        .find_one(doc! { "_id": &object_id }, None)
        .await?
        .ok_or_else(|| AppError::NotFound("Post not found after update".to_string()))?;

    Ok(Json(updated_post))
}

async fn delete_post(
    State(db): State<Database>,
    auth_user: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let posts_coll: Collection<BlogPost> = db.collection("posts");
    let object_id = id.parse::<mongodb::bson::oid::ObjectId>().map_err(|_| AppError::BadRequest("Invalid ID".to_string()))?;
    let post = posts_coll
        .find_one(doc! { "_id": &object_id }, None)
        .await?
        .ok_or_else(|| AppError::NotFound("Post not found".to_string()))?;

    if post.author_id.to_hex() != auth_user.0 {
        return Err(AppError::Auth("Not authorized to delete this post".to_string()));
    }

    posts_coll.delete_one(doc! { "_id": &object_id }, None).await?;
    Ok(StatusCode::NO_CONTENT)
}