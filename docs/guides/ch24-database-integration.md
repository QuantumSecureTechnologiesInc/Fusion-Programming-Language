# Chapter 24: Database Integration

Fusion provides excellent database integration through type-safe query builders, ORMs, and connection pooling. This chapter covers SQL databases, NoSQL databases, ORM patterns, and connection pooling.

## SQL Databases

### PostgreSQL

```fusion
use sqlx::postgres::PgPoolOptions;

#[derive(sqlx::FromRow, Debug)]
struct User {
    id: i64,
    name: String,
    email: String,
}

async fn postgres_example() -> Result<(), sqlx::Error> {
    // Create connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://user:password@localhost/database")
        .await?;
    
    // Execute raw query
    sqlx::query("CREATE TABLE IF NOT EXISTS users (id SERIAL PRIMARY KEY, name VARCHAR(100), email VARCHAR(255))")
        .execute(&pool)
        .await?;
    
    // Insert with parameters
    sqlx::query("INSERT INTO users (name, email) VALUES ($1, $2)")
        .bind("Alice")
        .bind("alice@example.com")
        .execute(&pool)
        .await?;
    
    // Query single row
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(1)
        .fetch_optional(&pool)
        .await?;
    
    println!("User: {:?}", user);
    
    // Query multiple rows
    let users = sqlx::query_as::<_, User>("SELECT * FROM users")
        .fetch_all(&pool)
        .await?;
    
    println!("Users: {:?}", users);
    
    Ok(())
}
```

### MySQL

```fusion
use sqlx::mysql::MySqlPool;

async fn mysql_example() -> Result<(), sqlx::Error> {
    let pool = MySqlPool::connect("mysql://user:password@localhost/database").await?;
    
    // Create table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id INT AUTO_INCREMENT PRIMARY KEY,
            name VARCHAR(100),
            email VARCHAR(255)
        )"
    )
    .execute(&pool)
    .await?;
    
    // Insert
    sqlx::query("INSERT INTO users (name, email) VALUES (?, ?)")
        .bind("Bob")
        .bind("bob@example.com")
        .execute(&pool)
        .await?;
    
    // Query
    let users = sqlx::query_as::<_, User>("SELECT * FROM users")
        .fetch_all(&pool)
        .await?;
    
    Ok(())
}
```

### SQLite

```fusion
use sqlx::sqlite::SqlitePool;

async fn sqlite_example() -> Result<(), sqlx::Error> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;
    
    // Create table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT,
            email TEXT
        )"
    )
    .execute(&pool)
    .await?;
    
    // Insert
    sqlx::query("INSERT INTO users (name, email) VALUES (?, ?)")
        .bind("Charlie")
        .bind("charlie@example.com")
        .execute(&pool)
        .await?;
    
    // Query
    let users = sqlx::query_as::<_, User>("SELECT * FROM users")
        .fetch_all(&pool)
        .await?;
    
    Ok(())
}
```

## NoSQL Databases

### MongoDB

```fusion
use mongodb::{Client, Collection, bson::doc};

#[derive(Debug, Serialize, Deserialize)]
struct User {
    name: String,
    email: String,
    age: Option<u32>,
}

async fn mongodb_example() -> Result<(), mongodb::error::Error> {
    let client = Client::with_uri_str("mongodb://localhost:27017").await?;
    let db = client.database("myapp");
    let collection: Collection<User> = db.collection("users");
    
    // Insert document
    let user = User {
        name: "Alice".into(),
        email: "alice@example.com".into(),
        age: Some(30),
    };
    
    collection.insert_one(user).await?;
    
    // Find documents
    let filter = doc! { "age": { "$gte": 25 } };
    let mut cursor = collection.find(filter).await?;
    
    while let Some(result) = cursor.next().await {
        let user = result?;
        println!("User: {:?}", user);
    }
    
    // Update document
    let filter = doc! { "name": "Alice" };
    let update = doc! { "$set": { "age": 31 } };
    collection.update_one(filter, update).await?;
    
    // Delete document
    let filter = doc! { "name": "Alice" };
    collection.delete_one(filter).await?;
    
    Ok(())
}
```

### Redis

```fusion
use redis::{Client, Commands};

fn redis_example() -> Result<(), redis::RedisError> {
    let client = Client::open("redis://127.0.0.1/")?;
    let mut con = client.get_connection()?;
    
    // String operations
    con.set("key", "value")?;
    let value: String = con.get("key")?;
    println!("Value: {}", value);
    
    // Hash operations
    con.hset("user:1", "name", "Alice")?;
    con.hset("user:1", "email", "alice@example.com")?;
    let name: String = con.hget("user:1", "name")?;
    println!("Name: {}", name);
    
    // List operations
    con.rpush("queue", "task1")?;
    con.rpush("queue", "task2")?;
    let task: String = con.lpop("queue", None)?;
    println!("Task: {}", task);
    
    // Set operations
    con.sadd("tags", "rust")?;
    con.sadd("tags", "fusion")?;
    let tags: Vec<String> = con.smembers("tags")?;
    println!("Tags: {:?}", tags);
    
    Ok(())
}
```

### DynamoDB

```fusion
use aws_sdk_dynamodb::{Client, types::AttributeValue};

async fn dynamodb_example() -> Result<(), Box<dyn std::error::Error>> {
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);
    
    // Put item
    let request = client.put_item()
        .table_name("Users")
        .item("id", AttributeValue::S("user1".into()))
        .item("name", AttributeValue::S("Alice".into()))
        .item("email", AttributeValue::S("alice@example.com".into()))
        .item("age", AttributeValue::N("30".into()));
    
    request.send().await?;
    
    // Get item
    let request = client.get_item()
        .table_name("Users")
        .key("id", AttributeValue::S("user1".into()));
    
    let response = request.send().await?;
    
    if let Some(item) = response.item {
        let name = item.get("name").and_then(|v| v.as_s().ok()).unwrap();
        println!("Name: {}", name);
    }
    
    // Query
    let request = client.query()
        .table_name("Users")
        .key_condition_expression("age >= :min_age")
        .expression_attribute_values(":min_age", AttributeValue::N("25".into()));
    
    let response = request.send().await?;
    
    if let Some(items) = response.items {
        for item in items {
            println!("User: {:?}", item);
        }
    }
    
    Ok(())
}
```

## ORM Patterns

### Diesel-style ORM

```fusion
use diesel::prelude::*;

table! {
    users (id) {
        id -> Int4,
        name -> Varchar,
        email -> Varchar,
    }
}

#[derive(Queryable, Selectable, AsChangeset, Debug)]
#[diesel(table_name = users)]
struct User {
    id: i32,
    name: String,
    email: String,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = users)]
struct NewUser {
    name: String,
    email: String,
}

impl User {
    fn create(conn: &mut PgConnection, new_user: NewUser) -> QueryResult<User> {
        diesel::insert_into(users::table)
            .values(&new_user)
            .returning(User::as_returning())
            .get_result(conn)
    }
    
    fn find(conn: &mut PgConnection, user_id: i32) -> QueryResult<User> {
        users::table.find(user_id).first(conn)
    }
    
    fn find_by_email(conn: &mut PgConnection, user_email: &str) -> QueryResult<User> {
        users::table.filter(users::email.eq(user_email)).first(conn)
    }
    
    fn all(conn: &mut PgConnection) -> QueryResult<Vec<User>> {
        users::table.load(conn)
    }
    
    fn update(conn: &mut PgConnection, user_id: i32, name: &str) -> QueryResult<User> {
        diesel::update(users::table.find(user_id))
            .set(users::name.eq(name))
            .returning(User::as_returning())
            .get_result(conn)
    }
    
    fn delete(conn: &mut PgConnection, user_id: i32) -> QueryResult<usize> {
        diesel::delete(users::table.find(user_id)).execute(conn)
    }
}
```

### SeaORM

```fusion
use sea_orm::{entity::prelude::*, ActiveModelTrait, Set};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub email: String,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::posts::Entity")]
    Posts,
}

impl Related<super::posts::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Posts.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Entity {
    pub async fn find_by_id(db: &DatabaseConnection, id: i32) -> Result<Option<Model>, DbErr> {
        Entity::find_by_id(id).one(db).await
    }
    
    pub async fn find_all(db: &DatabaseConnection) -> Result<Vec<Model>, DbErr> {
        Entity::find().all(db).await
    }
    
    pub async fn create(db: &DatabaseConnection, name: String, email: String) -> Result<Model, DbErr> {
        let active_model = ActiveModel {
            name: Set(name),
            email: Set(email),
            ..Default::default()
        };
        
        active_model.insert(db).await
    }
    
    pub async fn update_name(db: &DatabaseConnection, id: i32, name: String) -> Result<Model, DbErr> {
        let mut active_model = Entity::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("User not found".into()))?
            .into_active_model();
        
        active_model.name = Set(name);
        
        active_model.update(db).await
    }
    
    pub async fn delete(db: &DatabaseConnection, id: i32) -> Result<DeleteResult, DbErr> {
        Entity::delete_by_id(id).exec(db).await
    }
}
```

## Connection Pooling

### Built-in Connection Pool

```fusion
use sqlx::Pool;
use sqlx::postgres::PgPool;

struct Database {
    pool: PgPool,
}

impl Database {
    async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = Pool::connect(database_url).await?;
        
        // Run migrations
        sqlx::migrate!().run(&pool).await?;
        
        Ok(Self { pool })
    }
    
    async fn get_user(&self, id: i64) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }
    
    async fn create_user(&self, name: &str, email: &str) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING *"
        )
        .bind(name)
        .bind(email)
        .fetch_one(&self.pool)
        .await
    }
    
    async fn update_user(&self, id: i64, name: &str) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "UPDATE users SET name = $1 WHERE id = $2 RETURNING *"
        )
        .bind(name)
        .bind(id)
        .fetch_one(&self.pool)
        .await
    }
    
    async fn delete_user(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
}

// Usage
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::new("postgres://user:password@localhost/database").await?;
    
    // Create user
    let user = db.create_user("Alice", "alice@example.com").await?;
    println!("Created user: {:?}", user);
    
    // Get user
    let user = db.get_user(user.id).await?;
    println!("Got user: {:?}", user);
    
    // Update user
    let user = db.update_user(user.id, "Alice Smith").await?;
    println!("Updated user: {:?}", user);
    
    // Delete user
    db.delete_user(user.id).await?;
    println!("Deleted user");
    
    Ok(())
}
```

### Custom Connection Pool

```fusion
use std::sync::Arc;
use tokio::sync::Mutex;

struct ConnectionPool<T> {
    connections: Arc<Mutex<Vec<T>>>,
    max_size: usize,
}

impl<T> ConnectionPool<T> {
    fn new(max_size: usize) -> Self {
        Self {
            connections: Arc::new(Mutex::new(Vec::with_capacity(max_size))),
            max_size,
        }
    }
    
    async fn get(&self) -> Option<T> {
        let mut connections = self.connections.lock().await;
        connections.pop()
    }
    
    async fn put(&self, connection: T) {
        let mut connections = self.connections.lock().await;
        if connections.len() < self.max_size {
            connections.push(connection);
        }
    }
    
    async fn with_connection<F, R>(&self, f: F) -> Result<R, PoolError>
    where
        F: std::future::Future<Output = Result<R, PoolError>>,
    {
        let connection = self.get().await.ok_or(PoolError::ConnectionUnavailable)?;
        
        let result = f(connection).await;
        
        // Note: This is simplified. In production, you'd need to return the connection.
        // The actual implementation would use a guard or callback pattern.
        
        result
    }
}
```

## Transactions

### Transaction Management

```fusion
use sqlx::Transaction;
use sqlx::Postgres;

async fn transfer_money(
    pool: &PgPool,
    from_id: i64,
    to_id: i64,
    amount: i64,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    
    // Debit sender
    let sender = sqlx::query_as::<_, User>(
        "UPDATE users SET balance = balance - $1 WHERE id = $2 RETURNING *"
    )
    .bind(amount)
    .bind(from_id)
    .fetch_one(&mut *tx)
    .await?;
    
    if sender.balance < 0 {
        tx.rollback().await?;
        return Err(sqlx::Error::RowNotFound);
    }
    
    // Credit receiver
    sqlx::query(
        "UPDATE users SET balance = balance + $1 WHERE id = $2"
    )
    .bind(amount)
    .bind(to_id)
    .execute(&mut *tx)
    .await?;
    
    // Commit transaction
    tx.commit().await?;
    
    Ok(())
}
```

### Nested Transactions

```fusion
async fn complex_operation(pool: &PgPool) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    
    // First operation
    sqlx::query("INSERT INTO logs (message) VALUES ($1)")
        .bind("Operation started")
        .execute(&mut *tx)
        .await?;
    
    // Nested transaction
    {
        let mut inner_tx = pool.begin().await?;
        
        sqlx::query("UPDATE counters SET value = value + 1 WHERE name = $1")
            .bind("operation_count")
            .execute(&mut *inner_tx)
            .await?;
        
        inner_tx.commit().await?;
    }
    
    // Second operation
    sqlx::query("INSERT INTO logs (message) VALUES ($1)")
        .bind("Operation completed")
        .execute(&mut *tx)
        .await?;
    
    tx.commit().await?;
    
    Ok(())
}
```

## Summary

Fusion's database integration features include:

1. **SQL Databases**: PostgreSQL, MySQL, SQLite with type-safe queries
2. **NoSQL Databases**: MongoDB, Redis, DynamoDB support
3. **ORM Patterns**: Diesel and SeaORM for object-relational mapping
4. **Connection Pooling**: Built-in and custom pool implementations
5. **Transactions**: ACID compliance with transaction management

Fusion's type system ensures that database queries are type-safe, while async/await provides efficient handling of database operations.

In the next chapter, we'll explore cloud and DevOps practices with Fusion.