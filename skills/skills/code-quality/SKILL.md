---
name: code-quality
description: "The rules for the parameters, the properties and the return type of a signature. Use when you write a new method, a new function, a new class, a new constructor or a new property, when you add a parameter to a signature that exists, when you change what a method takes or what it returns, when you let a caller choose a value that a method holds today, when a method can find nothing and you choose what it gives back, when you decide that a value can be absent, when you decide how a class receives a dependency, when a parameter of type `bool` selects a behaviour, when a caller can turn a dependency such as a cache off, when you choose the type of a parameter or of a return type, when you write a cast, when a return type depends on an argument, when a caller outside this project can reach a type or a member, and when a signature carries `mixed`, `any`, an `array` with no shape, `= null`, `= []`, `= 0`, `= true`, `?`, `| null`, `| undefined`, `Option` or `Optional`."
---

# Code Quality

Obey each rule of this file each time that you write code and each time that you change code.

## 1. Write the type of each value in each signature

Write the type of the value in each signature that the value passes through: a parameter, a return type, a property, and the value of a dictionary.

Write no type of this table in a signature that you write, and write no dictionary of that type.

| The language | The type that carries no evidence |
| --- | --- |
| PHP | `mixed`, `object`, an `array` with no shape |
| TypeScript | `any`, `unknown`, `object`, `{}`, `Record<string, unknown>` |
| Java | `Object`, a raw `Map` |
| Kotlin | `Any` |
| C# | `object`, `dynamic` |
| Swift | `Any`, `AnyObject` |
| Rust | `Box<dyn Any>`, `serde_json::Value` |
| Python | `Any`, `dict[str, Any]` |
| Go | `any`, `interface{}`, `map[string]any` |
| Ruby | a `Hash` in the place of an object |

Write no cast and no assertion of a type. A cast repairs the evidence that a line above it threw away, thus delete that line. Two casts in one expression repair one fault two times.

Turn the strict mode of the language on in each file that you write, with the line of this table.

| The language | The line at the top of the file |
| --- | --- |
| PHP | `declare(strict_types=1);` |
| JavaScript | `'use strict';`, in a file that no module makes strict |
| Ruby, in a project that holds Sorbet | `# typed: strict` |

A language that is not in this table holds its strict mode in a configuration file. Call the Skill tool with "code-tooling" for that file.

Write a type parameter for a return type that depends on an argument. Give that parameter in the signature in a language that holds generics, such as `<TResult>`. Give it in the annotation of the static analyser in a language that holds none, such as `@template TResult` above the signature and `@return TResult` above a return type of `mixed`.

Write the shape of an array and the type parameter of a generic type in that annotation, in a language whose signature cannot hold them, such as `@return array<string, int>` above a return type of `array`, `@param list<string> $names` above a parameter of type `array`, `@var list<string>` above a property of type `array`, and `@return Collection<int, Post>` above a return type of `Collection`. That annotation is the one exception to the first table of this section, because it carries the evidence that the signature cannot hold. Write no other text in that annotation.

## 2. Write no nullable and no default value

Give each parameter, each property and each return type one state.

A nullable type carries two states: the value, and the absence of the value. A default value carries two states: the value that the caller gave, and the value that the signature holds. Each state is a branch, and each branch needs its own test, thus one function with two default values needs four tests.

Delete each default value from the signature, and give the value at each call site. Delete each nullable type, and use the replacement of section 3.

## 3. Replace a nullable and a default value

| The code holds | Write instead |
| --- | --- |
| A parameter with a default value | The value at each call site |
| A parameter that accepts null to select a behaviour | Two methods with two names |
| A collection that is null when it holds nothing | An empty collection |
| A property that is null until a later step | A second type that holds the value, and a constructor that takes it |
| A method that returns null when it finds nothing | One method that throws, and one method that returns a collection |
| A field that is null for one kind of record | A second type for that kind |
| A parameter that carries a new instance of a dependency as its default | A constructor that takes the dependency, and a factory that gives it |
| A parameter of type `bool` that turns a dependency off | A null object, section 4 |

Resolve a dependency one time, in the factory of the class or in the place that builds the program, and let the constructor take that dependency. A test gives its own instance to that constructor, thus the signature needs no default value for it.

Give each new type a name that says the state that it holds, such as `DraftInvoice` and `SentInvoice`. A name and a type stay correct, and a branch does not.

## 4. Replace a flag argument with a null object

A parameter of type `bool` that turns a dependency off is a conditional in the form of an argument. The method holds two paths, one for each value of the flag, and each path needs its own test.

Delete the parameter. Write a second implementation of the dependency that answers each call with a miss: a read finds nothing, a write keeps nothing, and a count is zero. Let the constructor take the dependency, and let the caller that wants the dependency off give that implementation. The method then holds one path, and no test of the method reads the flag.

Give the implementation a name that says what it does, such as `NoCache`. The pattern has the name Null Object, and the two refactorings have the names Replace Conditional with Polymorphism and Remove Flag Argument.

## 5. Mark each item that is not the public API

Read `.hod/PROJECT.md` before you write a new type and before you change the visibility of a type. Take two facts from that file: whether this project is a package that a different developer installs or an application, and the file that holds the public API.

Give the public API of a package one place, and mark each other type of that package with the marker of this table. A reader thus sees the boundary in the file, and a change to a type behind that boundary breaks no caller.

A new type of a package is not the public API unless `.hod/PROJECT.md` names it there, thus that type carries the marker.

| The language | The marker of a type that is not the public API |
| --- | --- |
| PHP | `@internal` |
| TypeScript | a type that the entry point of the package does not export |
| Java | a package private type |
| Kotlin | `internal` |
| C# | `internal` |
| Swift | `internal` |
| Rust | `pub(crate)` |
| Python | a name with the prefix `_`, and an `__all__` that omits that name |
| Go | a name that starts with a small letter, or the directory `internal/` |
| Ruby | `private_constant` |

Give each member the narrowest visibility that its callers need. Write each method private, and make it public at the first caller outside its own type. A member that one method of the same type calls stays private.

Give the constructor the visibility `private` when one static method of the same type is the one place that builds the object, and give that method a name that says what it builds, such as `create`.

## 6. Keep the public API of a package compatible

Accept a default value in the public API of a package when you add a parameter to a signature that a caller outside this repository already calls. A new required parameter breaks that caller, and a default value keeps the version compatible.

Write no default value in a new signature of that public API. No caller outside this repository calls a new signature, thus no caller breaks.

The rule for a nullable does not change in the public API of a package. Section 3 gives the replacement of each one.

Each other file is internal: the code of an application, and the code below the public API of a package. Write no nullable and no default value in internal code.

## 7. Stop when no replacement of section 3 fits

Stop when the code is internal and no line of section 3 replaces the nullable or the default value.

Give the user the path of the file, the signature, each state that the signature carries, and the reason that each line of section 3 fails. Ask the user to accept the nullable or the default value, and wait for the answer.

Weigh one exception on its own. An exception that the user accepted for one signature gives no exception for a different signature.

## 8. Report

Run the tests of this project after you change a signature.

Name each type of the first table of section 1 that you deleted from a signature, and name the type that you wrote in its place.

Name each cast that you deleted, and name the line that threw the evidence away.

Name each file where you wrote the line of the strict mode of section 1.

Name each type parameter and each shape of an array that you wrote, and name the signature that carries it.

Name each parameter that you made required, and say that it carries no default value.

Name the path of each file that holds a call site that you changed. Name each path, and not the number of them.

Name each nullable type that you deleted, and name the replacement of section 3 that you wrote in its place.

Name each parameter of type `bool` that you deleted, and name the null object that you wrote in its place.

Name `.hod/PROJECT.md`, say whether that file calls this project a package or an application, and name the file that holds the public API, each time that you wrote a type or changed a type.

Name each type that you marked with the marker of section 5, and name each member that you made private. Name each type that you left without that marker, and give the reason.

Say that you ran the tests of this project, and give the result of that run.

Write this report in the last message of your answer. Write no sentence for a rule of this file that your work did not touch.
