# placeholder-query (Name TBD)

placeholder-query is a set of libraries for querying databases and composing data fetches.

It draws on [Rel8](https://github.com/circuithub/rel8), [Haxl](https://github.com/facebook/Haxl) and [sofetch](https://github.com/iand675/sofetch).

## Schema

The following examples will use the schema below:

```rust
use placeholder_query_macro::{QueryInput, row, table};
use placeholder_query_postgres::{ProjectionExt, query::select as pg};

#[table(derive(Clone))]
pub struct User {
    pub id: i32,
    pub name: String,
}

#[table(derive(Clone, Debug))]
pub struct Post {
    pub id: i32,
    pub author_id: i32,
    pub title: String,
    pub body: String,
}

#[table]
pub struct Tag {
    pub id: i32,
    pub name: String,
}

#[table]
pub struct PostTag {
    pub post_id: i32,
    pub tag_id: i32,
}
```

The examples import `placeholder_query_postgres::query::select` as `pg`.
Calls such as `pg::from` and `pg::query` come from that module.

## Queries

### Select

```rust
let all_posts = pg::from(Post::table()).compile();
```

```sql
SELECT t0."id", t0."author_id", t0."title", t0."body" FROM "post" AS t0
```

### Partial select

```rust
#[row(derive(Clone, Debug))]
pub struct PostCard {
    pub id: i32,
    pub title: String,
}

let all_post_cards = pg::from(Post::table())
    .map(|post| PostCard {
        id: post.id,
        title: post.title,
    })
    .compile();
```

Only the columns used by the projection are selected:

```sql
SELECT t0."id", t0."title" FROM "post" AS t0
```

### Join

```rust
#[row(derive(Clone, Debug))]
pub struct PostWithAuthor {
    pub title: String,
    pub body: String,
    pub author_id: i32,
    pub author_name: String,
}

let posts_with_authors = pg::from(Post::table())
    .join(User::table(), |(post, author)| {
        post.author_id().eq(author.id())
    })
    .map(|(post, author)| PostWithAuthor {
        title: post.title,
        body: post.body,
        author_id: post.author_id,
        author_name: author.name,
    });
```

```sql
SELECT t0."title", t0."body", t0."author_id", t1."name"
FROM "post" AS t0
JOIN "user" AS t1 ON t0."author_id" = t1."id"
```

`join` can be chained, making it similar to `zip`.

```rust
#[row(derive(Clone, Debug))]
pub struct TaggedPost {
    pub title: String,
    pub tag_name: String,
}

let tagged_posts = pg::from(Post::table())
    .join(PostTag::table(), |(post, post_tag)| {
        post.id().eq(post_tag.post_id())
    })
    .join(Tag::table(), |((_, post_tag), tag)| {
        post_tag.tag_id().eq(tag.id())
    })
    .map(|((post, _), tag)| TaggedPost {
        title: post.title,
        tag_name: tag.name,
    })
    .compile();
```

```sql
SELECT t0."title", t2."name"
FROM "post" AS t0
JOIN "post_tag" AS t1 ON t0."id" = t1."post_id"
JOIN "tag" AS t2 ON t1."tag_id" = t2."id"
```

#### Lateral join

Use `lateral` when the joined query needs columns from the current row.

```rust
#[row(derive(Clone, Debug))]
pub struct LatestPostByAuthor {
    pub author_name: String,
    pub title: String,
}

let latest_posts_by_author = pg::from(User::table())
    .lateral(|author| {
        pg::from(Post::table())
            .filter(|post| post.author_id().eq(author.id()))
            .order_by(|post| post.id().desc())
            .limit(1)
            .map(|post| LatestPostByAuthor {
                author_name: author.name,
                title: post.title,
            })
    })
    .compile();
```

```sql
SELECT t1.c0, t1.c1
FROM "user" AS t0
CROSS JOIN LATERAL (
    SELECT t0."name", t2."title"
    FROM "post" AS t2
    WHERE t2."author_id" = t0."id"
    ORDER BY t2."id" DESC
    LIMIT 1
) AS t1(c0, c1)
```

### Transform results

### Subqueries

`pg::from` accepts another query as a subquery.

```rust
let post_titles = pg::from(posts_with_authors.clone())
    .order_by(|post| post.title().asc())
    .map(|post| post.title)
    .compile();
```

```sql
SELECT t0.c0
FROM (
    SELECT t1."title", t1."body", t1."author_id", t2."name"
    FROM "post" AS t1
    JOIN "user" AS t2 ON t1."author_id" = t2."id"
) AS t0(c0, c1, c2, c3)
ORDER BY t0.c0
```

#### Expressions

Use `Select::map` to transform SQL expressions.

```rust
use placeholder_query_postgres::query::mode;

#[derive(QueryInput)]
pub struct ListPosts {
    pub viewer_id: i32,
}

#[row]
pub struct PostListItem {
    pub title: String,
    pub author: String,
    pub is_mine: bool,
}

let post_list = pg::query::<ListPosts, _>(|input| {
    pg::from(posts_with_authors.clone())
        .map(|post| PostListItem::<mode::Expr> {
            title: post.title,
            author: post.author_name,
            is_mine: post.author_id.eq(input.viewer_id),
        })
        .order_by(|post| post.title().asc())
})
.compile();
```

```sql
SELECT t0.c0, t0.c3, t0.c2 = $1
FROM (
    SELECT t1."title", t1."body", t1."author_id", t2."name"
    FROM "post" AS t1
    JOIN "user" AS t2 ON t1."author_id" = t2."id"
) AS t0(c0, c1, c2, c3)
ORDER BY t0.c0
```

#### Values

Use `ProjectionExt::map` to transform the result.

```rust
#[derive(Debug)]
pub struct PostView {
    pub title: String,
    pub author: String,
    pub preview: String,
    pub reading_minutes: usize,
}

let post_views = pg::from(posts_with_authors.clone())
    .order_by(|post| post.title().asc())
    .map(|post| {
        post.map(|post: PostWithAuthor| PostView {
            title: post.title,
            author: post.author_name,
            preview: post.body.chars().take(160).collect(),
            reading_minutes: post.body.split_whitespace().count().div_ceil(200),
        })
    })
    .compile();
```

```sql
SELECT t0.c0, t0.c1, t0.c2, t0.c3
FROM (
    SELECT t1."title", t1."body", t1."author_id", t2."name"
    FROM "post" AS t1
    JOIN "user" AS t2 ON t1."author_id" = t2."id"
) AS t0(c0, c1, c2, c3)
ORDER BY t0.c0
```

### Parameters

```rust
#[derive(QueryInput)]
pub struct PostsByAuthor {
    pub author_id: i32,
}

let posts_by_author = pg::query::<PostsByAuthor, _>(|input| {
    pg::from(Post::table()).filter(|post| post.author_id().eq(input.author_id))
})
.compile();
```

```sql
SELECT t0."id", t0."author_id", t0."title", t0."body"
FROM "post" AS t0
WHERE t0."author_id" = $1
```

### Batch queries with UNNEST

`pg::query::<[T], _>` creates array parameters.

Use `unnest()` to convert them into rows.

```rust
#[derive(QueryInput, Clone, PartialEq, Eq, Hash)]
pub struct PostsByTag {
    pub tag_id: i32,
}

let tag_posts = pg::query::<[PostsByTag], _>(|reqs| {
    pg::from(reqs.unnest())
        .join(PostTag::table(), |(req, post_tag)| {
            req.tag_id.clone().eq(post_tag.tag_id())
        })
        .join(Post::table(), |((_, post_tag), post)| {
            post_tag.post_id().eq(post.id())
        })
        .order_by(|(_, post)| post.id().asc())
        .map(|((req, _), post)| (req, post))
})
.compile();
```

```sql
SELECT t0."tag_id", t2."id", t2."author_id", t2."title", t2."body"
FROM unnest($1::integer[]) AS t0("tag_id")
JOIN "post_tag" AS t1 ON t0."tag_id" = t1."tag_id"
JOIN "post" AS t2 ON t1."post_id" = t2."id"
ORDER BY t2."id"
```

## Fetch

Fetch is a scheduler similar to Haxl.

### API reference

```rust
impl<E, A> Fetch<E, A> {
    pub fn map<B>(self, f: impl FnOnce(A) -> B) -> Fetch<E, B>;
    pub fn and_then<B>(self, f: impl FnOnce(A) -> Fetch<E, B>) -> Fetch<E, B>;
    pub fn zip<B>(self, other: Fetch<E, B>) -> Fetch<E, (A, B)>;
}

pub fn traverse<E, T, B>(
    items: impl IntoIterator<Item = T>,
    f: impl Fn(T) -> Fetch<E, B>,
) -> Fetch<E, Vec<B>>;
```

The next example first loads posts for a tag, then loads each post author.

```rust
use placeholder_query_runtime::{Request, fetch, traverse};

impl Request for PostsByTag {
    type Output = Vec<Post>;
}

#[derive(QueryInput, Clone, PartialEq, Eq, Hash)]
pub struct UserById {
    pub id: i32,
}

impl Request for UserById {
    type Output = User;
}

pub struct FeedPost {
    pub post: Post,
    pub author: User,
}

let feed_posts = fetch(PostsByTag { tag_id: 1 }).and_then(|posts| {
    traverse(posts, |post| {
        fetch(UserById { id: post.author_id })
            .map(|author| FeedPost { post, author })
    })
});
```

The viewer and feed posts can load independently, so `zip` combines them.
Then `map` builds the final value.

```rust
pub struct TagFeed {
    pub viewer: User,
    pub posts: Vec<FeedPost>,
}

let tag_feed = fetch(UserById { id: 42 })
    .zip(feed_posts)
    .map(|(viewer, posts)| TagFeed { viewer, posts });
```

Requests are batched and deduplicated during execution. 

Results preserve the input order and are cached for the duration of a single run.

## Interpreting with tokio-postgres

`Executor::run` can execute compiled queries and fetch computations.

```rust
use placeholder_query_tokio_postgres::Executor;
use tokio_postgres::NoTls;

// ...
let (client, connection) = tokio_postgres::connect(&database_url, NoTls).await?;
// ...
let executor = Executor::new(client);

// run the compiled query
let posts: Vec<Post> = executor.run(all_posts).await?;
```

### Fetch requests

Implement `DataSource<T>` for the execution environment to interpret a request:

```rust
use std::collections::HashMap;
use placeholder_query_runtime::DataSource;
use placeholder_query_tokio_postgres::Error;

impl DataSource<PostsByTag> for Executor {
    async fn fetch<'a>(
        &self,
        reqs: impl IntoIterator<Item = &'a PostsByTag>,
    ) -> Result<HashMap<PostsByTag, Vec<Post>>, Error> {
        let posts_by_tags = pg::query::<[PostsByTag], _>(|reqs| {
            pg::from(reqs.unnest())
                .join(PostTag::table(), |(req, post_tag)| {
                    req.tag_id.clone().eq(post_tag.tag_id())
                })
                .join(Post::table(), |((_, post_tag), post)| {
                    post_tag.post_id().eq(post.id())
                })
                .order_by(|(_, post)| post.id().asc())
                .map(|((req, _), post)| (req, post))
        })
        .compile();

        self.fetch_batch(&posts_by_tags, reqs).await
    }
}

```

Use `LazyLock` to build and compile a query once, then reuse it across batches:

```rust
use std::sync::LazyLock;
use placeholder_query_postgres::query::select::template::SelectTemplate;

impl DataSource<UserById> for Executor {
    async fn fetch<'a>(
        &self,
        reqs: impl IntoIterator<Item = &'a UserById>,
    ) -> Result<HashMap<UserById, User>, Error> {
        static USERS_BY_ID: LazyLock<
            SelectTemplate<(UserByIdColumns, User<mode::Expr>), [UserById]>,
        > = LazyLock::new(|| {
            pg::query::<[UserById], _>(|reqs| {
                pg::from(reqs.unnest())
                    .join(User::table(), |(req, user)| req.id().eq(user.id()))
                    .map(|(req, user)| (req, user))
            })
            .compile()
        });

        self.fetch_batch(&USERS_BY_ID, reqs).await
    }
}
```
