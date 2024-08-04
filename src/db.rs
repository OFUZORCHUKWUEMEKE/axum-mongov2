use mongodb::{Client, Database};

pub async fn connect_db() -> mongodb::error::Result<Database> {
    let client = Client::with_uri_str("mongodb://localhost:27017").await?;
    Ok(client.database("blog_db"))
}
