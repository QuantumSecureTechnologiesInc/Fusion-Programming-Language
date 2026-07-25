> **Phase 0 audit (2026-06-24) found this doc overclaims reality.**
> Treat feature lists here as roadmap, not current state.
> See `docs-truth-audit/TRUTH_REPORT.md` for details.

# Chapter 12: An I/O Project: Building a Command Line Program

This chapter is a recap of the many skills you’ve learned so far. We will build a command line tool that interacts with file input/output (I/O). We'll re-implement the classic unix tool `grep` (globally search a regular expression and print). We'll call it `minigrep`.

Our tool will take a file path and a string to search for, then print lines from the file that contain the string.

We will practice:
- Organizing code (Chapter 7)
- Using vectors and strings (Chapter 8)
- Handling errors (Chapter 9)
- Using traits and lifetimes (Chapter 10)
- Writing tests (Chapter 11)

---

## 12.1 Accepting Command Line Arguments

First, create a new project:

```bash
fusion new minigrep
cd minigrep
```text

We need to read the arguments passed to the program (e.g., `minigrep searchstring filename.txt`). We'll use `std::env::args`.

```fusion
use std::env

fn main() {
    let args: Vec<String> = env::args().collect()

    // args[0] is the program name itself
    let query = &args[1]
    let filename = &args[2]

    println("Searching for {}", query)
    println("In file {}", filename)
}
```text

Try running it:

```bash
fusion run -- test sample.txt
```text

The `--` separates arguments for fusion from arguments for your program.

### Problem: Variable Scope and Error Handling

If the user provides no arguments, this program will panic (index out of bounds). We should fix this.

---

## 12.2 Reading a File

Let's read the file specified.

```fusion
use std::env
use std::fs

fn main() {
    let args: Vec<String> = env::args().collect()

    let query = &args[1]
    let filename = &args[2]

    println!("Searching for {}", query)
    println!("In file {}", filename)

    let contents = fs::read_to_string(filename)
        .expect("Something went wrong reading the file")

    println!("With text:\n{}", contents)
}
```text

Create a file `poem.txt` with some text to test it.

---

## 12.3 Refactoring and Separation of Concerns

Our `main` function is managing argument parsing AND file reading AND logic. This violates the **Single Responsibility Principle**. As the program grows, this will become messy.

We should split our code into `main.fu` and `lib.fu`.

### 12.3.1 Extracting the Argument Parser

Let's clean up the argument parsing.

```fusion
struct Config {
    query: String,
    filename: String,
}

impl Config {
    fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("not enough arguments")
        }

        let query = args[1].clone()
        let filename = args[2].clone()

        Ok(Config { query, filename })
    }
}
```text

### 12.3.2 Updating main.fu

```fusion
use std::env
use std::process

fn main() {
    let args: Vec<String> = env::args().collect()

    let config = Config::new(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err) // Print to stderr
        process::exit(1)
    })

    println!("Searching for {}", config.query)
    println!("In file {}", config.filename)

    // ...
}
```text

### 12.3.3 Extracting the Logic to `lib.fu`

Move the `Config` struct and the `run` logic to `src/lib.fu`.

`src/lib.fu`:

```fusion
use std::error::Error
use std::fs

pub struct Config {
    pub query: String,
    pub filename: String,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("not enough arguments")
        }
        let query = args[1].clone()
        let filename = args[2].clone()
        Ok(Config { query, filename })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(&config.filename)?

    for line in search(&config.query, &contents) {
        println!("{}", line)
    }

    Ok(())
}

pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let mut results = Vec::new()

    for line in contents.lines() {
        if line.contains(query) {
            results.push(line)
        }
    }

    results
}
```text

`src/main.fu`:

```fusion
use std::env
use std::process
use minigrep::Config // Import from library crate

fn main() {
    let args: Vec<String> = env::args().collect()

    let config = Config::new(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err)
        process::exit(1)
    })

    if let Err(e) = minigrep::run(config) {
        eprintln!("Application error: {}", e)
        process::exit(1)
    }
}
```text

---

## 12.4 Developing the Library Functionality with TDD

Let's focus on the `search` function. We'll use Test Driven Development (TDD).

### Step 1: Write a Failing Test

In `src/lib.fu`:

```fusion

#[cfg(test)]

mod tests {
    use super::*

    #[test]
    fn one_result() {
        let query = "duct"
        let contents = "\
Rust:
safe, fast, productive.
Pick three."

        assert_eq!(vec!["safe, fast, productive."], search(query, contents))
    }
}
```text

Run `fusion test`. It should fail (because `search` isn't fully implemented or returns empty).

### Step 2: Write Code to Pass the Test

Implement `search` (as shown in the previous section):

```fusion
pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let mut results = Vec::new()
    for line in contents.lines() {
        if line.contains(query) {
            results.push(line)
        }
    }
    results
}
```text

Run `fusion test`. It passes!

### Note on Lifetimes

The signature `search<'a>(query: &str, contents: &'a str) -> Vec<&'a str>` is crucial.
It says: "The returned vector contains slices that reference the `contents` string." The `query` string doesn't need to live as long as the return value, but the `contents` do.

---

## 12.5 Working with Environment Variables

Let's add a feature for case-insensitive search that the user can turn on via an environment variable.

In `src/lib.fu`:

```fusion
use std::env

pub struct Config {
    pub query: String,
    pub filename: String,
    pub case_sensitive: bool,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &'static str> {
        // ...
        let case_sensitive = env::var("CASE_INSENSITIVE").is_err()

        Ok(Config { query, filename, case_sensitive })
    }
}

pub fn search_case_insensitive<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let query = query.to_lowercase()
    let mut results = Vec::new()

    for line in contents.lines() {
        if line.to_lowercase().contains(&query) {
            results.push(line)
        }
    }
    results
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string(&config.filename)?

    let results = if config.case_sensitive {
        search(&config.query, &contents)
    } else {
        search_case_insensitive(&config.query, &contents)
    }

    for line in results {
        println!("{}", line)
    }

    Ok(())
}
```text

Now run:

```bash
CASE_INSENSITIVE=1 fusion run -- to poem.txt
```text

---

## 12.6 Writing Messages to Standard Error

We used `eprintln!` instead of `println!` for errors.
- `stdout` (standard output) is for your program's actual output.
- `stderr` (standard error) is for error messages.

This allows users to pipe output to a file but still see errors on screen:

```bash
fusion run > output.txt
```text

Errors will still appear in the terminal, while the grep results go to `output.txt`.

---

## 12.7 Summary

You have built a robust CLI tool!
- Used **Result** for error handling.
- Used **eprintln!** for proper CLI behavior.
- Used **traits** (`Box<dyn Error>`) for flexibility.
- Used **lifetimes** to handle string slices safely.
- Used **TDD** to ensure correctness.
- Used **environment variables** for configuration.

This chapter represents a major milestone. You have moved from learning language features to building real software.

---

## 12.8 Exercises

1. **Line Numbers**: Modify `search` to return line numbers along with the text.
2. **Highlighting**: colorize the matched query string in the output (using ANSI escape codes).
3. **Stdin**: Modify the program so that if no filename is given, it reads from standard input (pipe).

---

[Next: Chapter 13 - Functional Language Features: Iterators and Closures →](./chapter-13-functional.md)