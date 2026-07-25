> **Phase 0 audit (2026-06-24) found this doc overclaims reality.**
> Treat feature lists here as roadmap, not current state.
> See `docs-truth-audit/TRUTH_REPORT.md` for details.

# Chapter 8: Common Collections

Fusion's standard library includes a number of very useful data structures called **collections**. Unlike the built-in array and tuple types, data in these collections is stored on the heap, which means the amount of data does not need to be known at compile time and can grow or shrink as the program runs.

Each kind of collection has different capabilities and costs, and choosing the right one is a skill you'll develop over time. In this chapter, we'll discuss the three most commonly used collections:

- **Vector (`Vec<T>`)**: allows you to store a variable number of values next to each other.
- **String (`String`)**: a collection of characters. We've used this before, but we'll discuss it in more depth.
- **Hash Map (`HashMap<K, V>`)**: allows you to associate a value with a particular key.

---

## 8.1 Storing Lists of Values with Vectors

The first collection type we’ll look at is `Vec<T>`, also known as a **vector**. Vectors allow you to store more than one value in a single data structure that puts all the values next to each other in memory. Vectors can only store values of the same type.

### 8.1.1 Creating a New Vector

```fusion
let v: Vec<i32> = Vec::new()
```text

Note that we added a type annotation here. Since we aren't inserting any values into this vector, Fusion doesn't know what kind of elements we intend to store.

If we insert values instantly, Fusion infers the type:

```fusion
let v = vec![1, 2, 3]
```text

The `vec!` macro is a convenient initialization shorthand.

### 8.1.2 Updating a Vector

To create a vector and then add elements to it, we can use the `push` method:

```fusion
let mut v = Vec::new()

v.push(5)
v.push(6)
v.push(7)
v.push(8)
```text

The variable `v` must be `mut`able to change it.

### 8.1.3 Reading Elements of Vectors

There are two ways to reference a value stored in a vector: via indexing syntax or the `get` method.

```fusion
let v = vec![1, 2, 3, 4, 5]

// Method 1: Indexing (panics if out of bounds)
let third: &i32 = &v[2]
println("The third element is {}", third)

// Method 2: Get Method (returns Option<&T>)
match v.get(2) {
    Some(third) => println("The third element is {}", third),
    None => println("There is no third element."),
}
```text

When handling user input or data where the index might be invalid, always use `.get()` to handle the error gracefully.

### 8.1.4 Ownership and Borrowing in Vectors

Remember the ownership rules: you cannot have mutable and immutable references to the same content at the same time.

```fusion
let mut v = vec![1, 2, 3, 4, 5]

let first = &v[0] // Immutable borrow

v.push(6) // Mutable borrow (potentially reallocates memory)

// println("The first element is: {}", first) // Error!
```text

Why? Adding a new element might require allocating new memory and copying the old elements to the new space (if there isn't enough capacity). If that happens, the reference `first` would point to deallocated memory. Fusion prevents this.

### 8.1.5 Iterating Over Values

To access each element in a vector in turn, we iterate through all of them.

```fusion
let v = vec![100, 32, 57]
for i in &v {
    println("{}", i)
}
```text

We can also iterate over mutable references to change values:

```fusion
let mut v = vec![100, 32, 57]
for i in &mut v {
    *i += 50 // Use dereference operator (*) to get to the value
}
```text

### 8.1.6 Using an Enum to Store Multiple Types

Vectors can only store values of the **same type**. If you need to store different types, you can define an enum!

```fusion
enum SpreadsheetCell {
    Int(i32),
    Float(f64),
    Text(String),
}

let row = vec![
    SpreadsheetCell::Int(3),
    SpreadsheetCell::Text(String::from("blue")),
    SpreadsheetCell::Float(10.12),
]
```text

This works because every element in the vector is of type `SpreadsheetCell`.

---

## 8.2 Storing UTF-8 Encoded Text with Strings

We discussed strings in Chapter 4, but let's go deeper. `String` is actually implemented as a wrapper around a `Vec<u8>` with extra guarantees about valid UTF-8 encoding.

### 8.2.1 Creating a New String

```fusion
let mut s = String::new()

let data = "initial contents"
let s = data.to_string()

let s = String::from("initial contents")
```text

### 8.2.2 Updating a String

You can grow a `String` by pushing more data into it.

```fusion
let mut s = String::from("foo")
s.push_str("bar") // Appends a string slice
s.push('!')       // Appends a single char
```text

You can also use the `+` operator or the `format!` macro for concatenation.

```fusion
let s1 = String::from("Hello, ")
let s2 = String::from("world!")
let s3 = s1 + &s2 // s1 is moved and can no longer be used
```text

For more complex formatting without taking ownership:

```fusion
let s1 = String::from("tic")
let s2 = String::from("tac")
let s3 = String::from("toe")

let s = format!("{}-{}-{}", s1, s2, s3)
```text

`format!` works like `println!` but returns a `String`.

### 8.2.3 Indexing into Strings

In many languages, you can access characters by index (`s[0]`). **In Fusion, you cannot.**

```fusion
let s = String::from("hello")
// let h = s[0] // Error!
```text

Why?
1. **UTF-8 Variable Width**: A character can be 1 to 4 bytes. Indexing `s[0]` on a multi-byte char (like `Здравствуйте`) would return the first byte, which is often not a valid character on its own.
2. **Performance expectation**: Indexing implies O(1) random access. But to find the Nth "character" in UTF-8, you must traverse the string, which is O(N).

### 8.2.4 Slicing Strings

If you really want a byte slice, use a range:

```fusion
let hello = "Здравствуйте"
let s = &hello[0..4] // Returns "Зд" (first 4 bytes)
```text

If you slice in the middle of a character boundary, Fusion will panic.

### 8.2.5 Iterating Over Strings

Be explicit about what you want: chars or bytes.

```fusion
for c in "Зд".chars() {
    println(c)
}
// Prints:
// З
// д

for b in "Зд".bytes() {
    println(b)
}
// Prints:
// 208
// 151
// ...
```text

---

## 8.3 Storing Keys with Associated Values in Hash Maps

The last of our common collections is the **hash map**. The type `HashMap<K, V>` stores a mapping of keys of type `K` to values of type `V` using a *hashing function*, which determines how it places these keys and values into memory.

### 8.3.1 Creating a New Hash Map

```fusion
use std::collections::HashMap

let mut scores = HashMap::new()

scores.insert(String::from("Blue"), 10)
scores.insert(String::from("Yellow"), 50)
```text

Like vectors, hashmaps store their data on the heap. All keys must have the same type, and all values must have the same type.

### 8.3.2 Accessing Values

```fusion
let team_name = String::from("Blue")
let score = scores.get(&team_name) // Returns Option<&i32>
```text

### 8.3.3 Hash Maps and Ownership

For types that implement the `Copy` trait, like `i32`, values are copied into the hash map. For owned values like `String`, the values will be moved and the hash map will be the owner of those values.

```fusion
let field_name = String::from("Favorite color")
let field_value = String::from("Blue")

let mut map = HashMap::new()
map.insert(field_name, field_value)
// field_name and field_value are invalid here
```text

### 8.3.4 Updating a Hash Map

**Overwriting a Value**:
If you insert a key that already exists, the old value is replaced.

```fusion
scores.insert(String::from("Blue"), 10)
scores.insert(String::from("Blue"), 25)
// "Blue" is now 25
```text

**Only Inserting If the Key Has No Value**:
We check if a key exists using `entry`.

```fusion
scores.entry(String::from("Yellow")).or_insert(50)
scores.entry(String::from("Blue")).or_insert(50)
```text

The return value of `entry` is an enum `Entry` that represents a value that might or might not exist. `or_insert` returns a mutable reference to the value if it exists, or inserts the parameter and returns a reference to the new value if it doesn't.

**Updating a Value Based on the Old Value**:

```fusion
let text = "hello world wonderful world"
let mut map = HashMap::new()

for word in text.split_whitespace() {
    let count = map.entry(word).or_insert(0)
    *count += 1
}

println!("{:?}", map)
// {"world": 2, "hello": 1, "wonderful": 1}
```text

### 8.3.5 Hashing Functions

By default, Fusion uses a cryptographically, security-hardened hashing function (SipHash) that provides resistance to Denial of Service (DoS) attacks. This is not the fastest hashing algorithm available, but the trade-off for better security is worth it. If you profile your code and find the default hash function is too slow, you can switch to another function by specifying a different *hasher*.

---

## 8.4 Summary

Vectors, strings, and hash maps will provide a large amount of functionality necessary for common programming tasks.
- `Vec<T>`: List of items.
- `String`: UTF-8 text (essentially `Vec<u8>`).
- `HashMap<K, V>`: Key-value store.

We will revisit collections when we look at **iterators** and **closures** in later chapters.

---

## 8.5 Exercises

1. **Median and Mode**: Given a list of integers, use a vector and calculate the median (when sorted, the value in the middle position) and mode (the value that occurs most often; a hash map will be helpful here).
2. **Pig Latin**: Convert strings to pig latin. The first consonant of each word is moved to the end of the word and "ay" is added, so "first" becomes "irst-fay". Words that start with a vowel have "hay" added to the end instead ("apple" becomes "apple-hay").
3. **Employee Directory**: Using a hash map and vectors, create a text interface to allow a user to add employee names to a department in a company. For example, "Add Sally to Engineering" or "Add Amir to Sales". Then let the user retrieve a list of all people in a department or all people in the company by department, sorted alphabetically.

---

[Next: Chapter 9 - Error Handling →](./chapter-09-error-handling.md)