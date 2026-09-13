use std::collections::HashMap;

use tokio_postgres::{Client, NoTls};

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub hair_color: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct Post {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
    pub body: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct Comment {
    pub id: i32,
    pub post_id: i32,
    pub text: String,
}

pub type Associations = Vec<(User, Vec<(Post, Vec<Comment>)>)>;

pub struct Fixture {
    pub users: Vec<User>,
    pub posts: Vec<Post>,
    pub comments: Vec<Comment>,
}

pub async fn connect(url: &str) -> Client {
    let (client, connection) = tokio_postgres::connect(url, NoTls).await.unwrap();
    tokio::spawn(async move { connection.await.unwrap() });
    client
}

pub async fn create_tables(client: &Client) {
    client
        .batch_execute(
            "CREATE TABLE IF NOT EXISTS users (
                id SERIAL PRIMARY KEY, name TEXT NOT NULL, hair_color TEXT
            );
            CREATE TABLE IF NOT EXISTS posts (
                id SERIAL PRIMARY KEY, user_id INTEGER NOT NULL REFERENCES users(id),
                title TEXT NOT NULL, body TEXT
            );
            CREATE TABLE IF NOT EXISTS comments (
                id SERIAL PRIMARY KEY, post_id INTEGER NOT NULL REFERENCES posts(id),
                text TEXT NOT NULL
            );",
        )
        .await
        .unwrap();
}

impl Fixture {
    pub fn trivial(size: i32) -> Self {
        Self {
            users: (1..=size)
                .map(|id| User {
                    id,
                    name: format!("User {}", id - 1),
                    hair_color: None,
                })
                .collect(),
            posts: Vec::new(),
            comments: Vec::new(),
        }
    }

    pub fn medium(size: i32) -> Self {
        let mut fixture = Self::trivial(size);
        for user in &mut fixture.users {
            user.hair_color = Some(if user.id % 2 == 1 { "black" } else { "brown" }.into());
        }
        fixture
    }

    pub fn associations() -> Self {
        let mut fixture = Self::medium(100);
        for user in &fixture.users {
            for index in 0..10 {
                let id = (user.id - 1) * 10 + index + 1;
                fixture.posts.push(Post {
                    id,
                    user_id: user.id,
                    title: format!("Post {index} by user {}", user.id),
                    body: None,
                });
                for index in 0..10 {
                    fixture.comments.push(Comment {
                        id: (id - 1) * 10 + index + 1,
                        post_id: id,
                        text: format!("Comment {index} on post {id}"),
                    });
                }
            }
        }
        fixture
    }

    pub async fn seed(&self, client: &Client) {
        client
            .batch_execute("BEGIN; TRUNCATE comments, posts, users RESTART IDENTITY;")
            .await
            .unwrap();
        let statement = client
            .prepare("INSERT INTO users (id, name, hair_color) VALUES ($1, $2, $3)")
            .await
            .unwrap();
        for user in &self.users {
            client
                .execute(&statement, &[&user.id, &user.name, &user.hair_color])
                .await
                .unwrap();
        }
        let statement = client
            .prepare("INSERT INTO posts (id, user_id, title, body) VALUES ($1, $2, $3, $4)")
            .await
            .unwrap();
        for post in &self.posts {
            client
                .execute(
                    &statement,
                    &[&post.id, &post.user_id, &post.title, &post.body],
                )
                .await
                .unwrap();
        }
        let statement = client
            .prepare("INSERT INTO comments (id, post_id, text) VALUES ($1, $2, $3)")
            .await
            .unwrap();
        for comment in &self.comments {
            client
                .execute(&statement, &[&comment.id, &comment.post_id, &comment.text])
                .await
                .unwrap();
        }
        client
            .batch_execute("COMMIT; ANALYZE users; ANALYZE posts; ANALYZE comments;")
            .await
            .unwrap();
    }

    pub fn assert_users(&self, mut users: Vec<User>) {
        users.sort_by_key(|user| user.id);
        assert_eq!(users, self.users);
    }

    pub fn assert_medium(&self, mut rows: Vec<(User, Option<Post>)>) {
        rows.sort_by_key(|(user, post)| (user.id, post.as_ref().map(|post| post.id)));
        let expected: Vec<_> = self
            .users
            .iter()
            .filter(|user| user.hair_color.as_deref() == Some("black"))
            .map(|user| (user.clone(), None))
            .collect();
        assert_eq!(rows, expected);
    }

    pub fn assert_associations(&self, mut rows: Associations) {
        rows.sort_by_key(|(user, _)| user.id);
        for (_, posts) in &mut rows {
            posts.sort_by_key(|(post, _)| post.id);
            for (_, comments) in posts {
                comments.sort_by_key(|comment| comment.id);
            }
        }
        let expected = self
            .users
            .iter()
            .map(|user| {
                let posts = self
                    .posts
                    .iter()
                    .filter(|post| post.user_id == user.id)
                    .map(|post| {
                        let comments = self
                            .comments
                            .iter()
                            .filter(|comment| comment.post_id == post.id)
                            .cloned()
                            .collect();
                        (post.clone(), comments)
                    })
                    .collect();
                (user.clone(), posts)
            })
            .collect::<Associations>();
        assert_eq!(rows, expected);
    }
}

pub fn group_associations(
    users: Vec<User>,
    posts: Vec<Post>,
    comments: Vec<Comment>,
) -> Associations {
    let mut comments_by_post = HashMap::<_, Vec<_>>::with_capacity(posts.len());
    for comment in comments {
        comments_by_post
            .entry(comment.post_id)
            .or_default()
            .push(comment);
    }
    let mut posts_by_user = HashMap::<_, Vec<_>>::with_capacity(users.len());
    for post in posts {
        let comments = comments_by_post.remove(&post.id).unwrap_or_default();
        posts_by_user
            .entry(post.user_id)
            .or_default()
            .push((post, comments));
    }
    users
        .into_iter()
        .map(|user| {
            let posts = posts_by_user.remove(&user.id).unwrap_or_default();
            (user, posts)
        })
        .collect()
}
