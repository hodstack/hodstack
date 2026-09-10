---
name: code-quality
description: "The rules for the parameters, the properties and the return type of a signature. Use when you write a new method, a new function, a new class, a new constructor or a new property, when you add a parameter to a signature that exists, when you change what a method takes or what it returns, when you let a caller choose a value that a method holds today, when a method can find nothing and you choose what it gives back, when you decide that a value can be absent, when you decide how a class receives a dependency, and when a signature carries `= null`, `= []`, `= 0`, `= true`, `?`, `| null`, `| undefined`, `Option` or `Optional`."
---

# Code Quality

Obey each rule of this file each time that you write code and each time that you change code.

## 1. Write no nullable and no default value

Give each parameter, each property and each return type one state.

A nullable type carries two states: the value, and the absence of the value. A default value carries two states: the value that the caller gave, and the value that the signature holds. Each state is a branch, and each branch needs its own test, thus one function with two default values needs four tests.

Delete each default value from the signature, and give the value at each call site. Delete each nullable type, and use the replacement of section 2.

## 2. Replace a nullable and a default value

| The code holds | Write instead |
| --- | --- |
| A parameter with a default value | The value at each call site |
| A parameter that accepts null to select a behaviour | Two methods with two names |
| A collection that is null when it holds nothing | An empty collection |
| A property that is null until a later step | A second type that holds the value, and a constructor that takes it |
| A method that returns null when it finds nothing | One method that throws, and one method that returns a collection |
| A field that is null for one kind of record | A second type for that kind |
| A parameter that carries a new instance of a dependency as its default | A constructor that takes the dependency, and a factory that gives it |

Resolve a dependency one time, in the factory of the class or in the place that builds the program, and let the constructor take that dependency. A test gives its own instance to that constructor, thus the signature needs no default value for it.

Give each new type a name that says the state that it holds, such as `DraftInvoice` and `SentInvoice`. A name and a type stay correct, and a branch does not.

## 3. Keep the public API of a package compatible

Read `.hod/PROJECT.md`. That file says what this project is: a package that a different developer installs, or an application.

Accept a default value in the public API of a package when you add a parameter to a signature that a caller outside this repository already calls. A new required parameter breaks that caller, and a default value keeps the version compatible.

Write no default value in a new signature of that public API. No caller outside this repository calls a new signature, thus no caller breaks.

The rule for a nullable does not change in the public API of a package. Section 2 gives the replacement of each one.

Each other file is internal: the code of an application, and the code below the public API of a package. Write no nullable and no default value in internal code.

## 4. Stop when no replacement of section 2 fits

Stop when the code is internal and no line of section 2 replaces the nullable or the default value.

Give the user the path of the file, the signature, each state that the signature carries, and the reason that each line of section 2 fails. Ask the user to accept the nullable or the default value, and wait for the answer.

Weigh one exception on its own. An exception that the user accepted for one signature gives no exception for a different signature.

## 5. Report

Run the tests of this project after you change a signature.

Name each parameter that you made required, and say that it carries no default value.

Name the path of each file that holds a call site that you changed. Name each path, and not the number of them.

Name each nullable type that you deleted, and name the replacement of section 2 that you wrote in its place.

Say that you ran the tests of this project, and give the result of that run.

Write this report in the last message of your answer. Write no sentence for a rule of this file that your work did not touch.
