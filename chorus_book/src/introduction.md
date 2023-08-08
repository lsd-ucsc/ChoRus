# Introduction

<p align="center">
  <img src="./assets/ChoRus.png" alt="ChoRus Logo" width="256" height="256">
</p>

Welcome to the ChoRus Book! This book is a guide to using the ChoRus library.

## What is ChoRus?

ChoRus is a library that enables [Choreographic Programming](https://en.wikipedia.org/wiki/Choreographic_programming) in Rust.

In distributed programming, it is often necessary to coordinate the behavior of multiple nodes. This coordination is typically achieved by writing a program that runs on each node. However, writing a program for each node can lead to bugs and inconsistencies.

Choreographic Programming is a programming paradigm that allows programmers to write "choreographies" that describe the desired behavior of a system as a whole. These choreographies can then be used to generate programs for each node in the system though a process called "end-point projection," or "EPP" for short.

## Choreographic Programming as a Library

In the past, choreographic programming has been implemented as a standard programming language. While this approach allows for flexibility for language designs, it makes it difficult to integrate choreographic programming into existing ecosystems.

ChoRus takes a different approach. Instead of implementing choreographic programming as a language, ChoRus implements choreographic programming as a library. ChoRus can be installed as a Cargo package and used in any Rust project. This allows choreographic programming to be used in any Rust project, including projects that use other libraries.

ChoRus is built on top of the "End-point Projection as Dependency Injection" (EPP-as-DI) approach. ChoRus also takes advantage of Rust's type system and macro system to provide a safe and ergonomic choreographic programming experience.

## Features

At high level, ChoRus provides the following features:

- Define choreographies by implementing the `Choreography` trait.
  - Passing located arguments to / receiving located return values from choreographies.
  - Location polymorphism.
  - Higher-order choreographies.
  - Efficient conditional with the `colocally` operator.
- Performing end-point projection.
- Pluggable message transports.
  - Two built-in transports: `Local` and `HTTP`.
  - Creating custom transports.
- Macros for defining locations and choreographies.
