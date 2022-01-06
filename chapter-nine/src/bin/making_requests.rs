use serde_derive::{Serialize, Deserialize};
use std::fmt;
use tokio;

#[derive(Serialize, Deserialize, Debug)]
// The JSON returned by the web service that hands posts out
// it written in camelCase, so we need to tell serde about that
#[serde(rename_all = "camelCase")]
struct Post {
    user_id: u32,
    id: u32,
    title: String,
    body: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct NewPost {
    user_id: u32,
    title: String,
    body: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
// The following struct could be rewritten with a builder
struct UpdatedPost {
    #[serde(skip_serializing_if = "Option::is_none")]
    user_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<String>,
}

struct PostCrud {
    client: reqwest::Client,
    endpoint: String,
}

impl fmt::Display for Post {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "User ID: {}\nID: {}\nTitle: {}\nBody: {}\n",
            self.user_id, self.id, self.title, self.body
        )
    }
}

impl PostCrud {
    fn new() -> Self {
        PostCrud {
            // Build an HTTP client. It's reusable!
            client: reqwest::Client::new(),
            // This is a link to a fake REST API service
            endpoint: "https://jsonplaceholder.typicode.com/posts".to_string(),
        }
    }

    async fn create(&self, post: &NewPost) -> Result<Post, reqwest::Error> {
        let response = self.client.post(&self.endpoint).json(post).send().await?.json().await?;
        Ok(response)
    }

    async fn read(&self, id: u32) -> Result<Post, reqwest::Error> {
        let url = format!("{}/{}", self.endpoint, id);
        let response = self.client.get(&url).send().await?.json().await?;
        Ok(response)
    }

    async fn update(&self, id: u32, post: &UpdatedPost) -> Result<Post, reqwest::Error> {
        let url = format!("{}/{}", self.endpoint, id);
        let response = self.client.patch(&url).json(post).send().await?.json().await?;
        Ok(response)
    }

    async fn delete(&self, id: u32) -> Result<(), reqwest::Error> {
        let url = format!("{}/{}", self.endpoint, id);
        self.client.delete(&url).send().await?;
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let post_crud = PostCrud::new();
    let post = post_crud.read(1).await.expect("Failed to read post");
    println!("Read a post:\n{}", post);

    let new_post = NewPost {
        user_id: 2,
        title: "Hello World!".to_string(),
        body: "This is a new post, sent to a fake JSON API server.\n".to_string(),
    };
    let post = post_crud.create(&new_post).await.expect("Failed to create post");
    println!("Created a post:\n{}", post);

    let updated_post = UpdatedPost {
        user_id: None,
        title: Some("New title".to_string()),
        body: None,
    };
    let post = post_crud
        .update(4, &updated_post)
        .await
        .expect("Failed to update post");
    println!("Updated a post:\n{}", post);

    post_crud.delete(51).await.expect("Failed to delete post");
}
