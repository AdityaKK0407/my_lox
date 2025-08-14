# my_lox

## Lox programming language from the book "Crafting Interpreters".

# Table of Contents

- [About](#about)
- [Features](#features)
- [How to Run](#how-to-run)
- [Sample Programs](#sample-programs)
- [Documentation](#documentation)
- [License](#license)
- [Feedback and Contributions](#feedback-and-contributions)

# About

This is a Rust implementation of the Lox language from the book _Crafting Interpreters_ by Robert Nystrom.
It is an interpreter that supports both _REPL_ and _running a `.lox` file_
This Lox version slightly differs from the original Lox.
To view the book, click [Crafting Interpreters](https://craftinginterpreters.com)

# Features

## REPL

- In REPL mode, semicolon is not required
- Every expression returns a value that is printed to console
- To exit a REPL, enter `exit`

## File

- All statements except if-else, loops, functions and classes must end with semicolon
- Command line arguments can be provided to main function

## Language

- All datatypes are immutable. A deep copy is created every time it is moved, passed to functions or returned from functions.
- No implicit typecasting.
- Strings follow ASCII standards. UTF and Non-UTF string types are not supported
- Builtin functions
  - clock - returns UNIX timestamp
  - scan - returns input given to console as a string
  - min - given a list of numbers, returns the minimum
  - max - given a list of numbers, returns the maximum
  - number - typecast variable to number
  - bool - typecast variable to bool
  - string - typecast variable to string
  - len - returns length of array or string
  - var_type - return the type of variable (even works for functions, classes and instances)
  - reverse - returns the reverse of string or array

# How to Run

You can run the interpreter in two ways:-

### Option 1: Prebuilt binary (Windows only)

Download the executable from the [Releases](https://github.com/AdityaKK0407/my_lox/releases) page and run it.

```bash
  # REPL mode
  lox

  # Run from a file
  lox file.lox [optional arguments to main function]
```

NOTE - This method only works for Windows OS. For Linux or Mac, use option 2.

### Option 2: Build from source (Cross-Platform)

- First you need Rust compiler. You can install from [Install Rust](https://www.rust-lang.org/tools/install)
- Run the following commands

  ```bash
    # Open a terminal (Command Prompt or PowerShell for Windows, Terminal for macOS or Linux)

    # Ensure git is installed on your system
    git clone https://github.com/AdityaKK0407/my_lox.git

    # Navigate to project directory
    cd lox

    # Build project (optional)
    cargo build --release
  ```

- Alternatively you can download the project from GitHub and save it in a directory of your choice
- To execute the program, run the following commands

  ```bash
      # REPL mode
      cargo run --release

      # Run from a file
      cargo run --release file.lox [optional arguments to main function]
  ```

- To install or uninstall the executable to system path, run the following commands

  ```bash
      # Build the executable
      cargo build --release

      # Install the executable to system path
      cargo install --path .

      # Uninstall executable
      cargo uninstall
  ```

- Now you will be able to directly run the executable from anywhere on your system

# Sample Programs

```javascript
  // main.lox

  fun greet(name) {
      println "Hello ", name, "!";
  }

  fun main() {
      var name = "Lox";
      greet(name);
   }
```

```bash
    # Option 1
    lox main.lox

    # Option 2
    cargo run -- main.lox
```

```text
    Hello Lox!
```

# Documentation

## Description of the language

- Datatypes

  - number - integer and float values

  ```javascript
  // 204 or 1.618
  ```

  - bool - true or false

  ```javascript
  // true or false
  ```

  - string - string values encased in either double quotes ("") or single quotes ('')

  ```javascript
  // "Hello World!" or 'Hello World!'
  ```

  - nil - null value

  ```javascript
  // nil
  ```

  - object - javascript object encased in curly braces ({})

  ```javascript
  // {
  //    a: 54,
  //    b,  // Only works if variable b is already declared
  // }
  ```

  - array - standard C (fixed-sized) array

  ```javascript
  // [1.618, "Lox", true]
  ```

- Operators

  - Arithmetic operators - (+, -, \*, /, %)
  - Logical operators - (and, or)
  - Comparison operators -(<, >, ==, !=, <=, >=)
  - Shorthand assignment operators - (+=, -=, \*=, /=, %=)
  - Unary operators - (-, !)

- Standard statements

  - print - used to write output to console
  - println - serves the same purpose as print but prints newline character at the end

  ```javascript
     print "Hello ", name, " to the world of Lox!";
     println "Hello ", name, " to the world of Lox!";
  ```

  - var - declaring variables
  - const - declaring constant variables

  ```javascript
  var name = "Aditya";
  const PHI = 1.618;
  ```

- Control flow statements

  - if/else - standard C style

  ```javascript
        if 3 > 2 {

        } else if 4 == 4  {

        } else {

        }
  ```

  - for - C style containing declaration, conditional statement and reassignment
    - NOTE - unlike in C, here all three statements namely (declaration, condition and reassignment) are necessary

  ```javascript
     for var i = 0; i < 5; i += 1 {}
  ```

  - while - standard C style. Only accepts bool values as condition

  ```javascript
      while true {

      }
  ```

  - break - early exit from loops
  - continue - start next iteration of loop and skip next lines of code inside loop
  - return - return a value or nil from functions or methods

- Functions

  - Declared using fun keyword.
  - Functions can be stored as variables, passed as parameters and returned from other functions.
  - Closures are also allowed

    ```javascript
    fun myFunc() {

    }
    ```

- Classes

  - Declared using class keyword
  - Methods and constructor have function syntax, using fun keyword
  - Constructors cannot return a value, they create an instance
  - Inheritance is supported using '<' operator
  - Single, Multilevel and Hierarchical inheritance are supported (Multiple inheritance is not supported)
  - Hybrid inheritance can also be formed using the allowed inheritances
  - this and super keywords are also supported and work as standard
  - Only static fields and methods are allowed inside classes

  ```javascript
      class A {
        var a = 100;
        fun A() {} // Constructor

        fun myMethod() {} // Method
      }
      class B < A {} // B inherits from A
  ```

## Error Handling

Errors are reported in the console with a clear message and the line number where they occurred.  
Certain low-level issues, such as memory overflows or infinite stack traces, are handled by the Rust runtime rather than the interpreter itself.

# License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.

# Feedback and Contributions

I’ve worked to include all the essential features and make the language as robust as possible.  
If you spot bugs, have suggestions for improvements, or want to request new features, please share your feedback.

You can contribute by opening an issue or joining the discussions.  
Your support is greatly appreciated, and I look forward to hearing your thoughts.
