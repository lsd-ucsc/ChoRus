# ChoRus Examples

This directory contains examples of how to use the ChoRus library. You can run them with `cargo run --example <example_name>`.

## `bookseller.rs`

This example shows how to implement the bookseller protocol in ChoRus. On top of the file, you can find the `get_book` function that defines the list of books and their prices and delivery dates. `BUDGET` defines the budget of the client.

The `main` function executes the choreography using `LocalTransport`. You can type the name of the book (`"TAPL"` or `"HoTT"`) and it will print the delivery date if the price is within the budget.

## `bookseller2.rs`

This example implements the same bookseller protocol with some twists. Now, the entry choreography `BooksellerChoreography` is higher-order and receives another choreography `Decider` as a parameter. The `Decider` choreography is used to decide whether `Buyer1` purchases the book. In the same file, you can find two implementations of `Decider`: `OneBuyerDecider` and `TwoBuyerDecider`. As the names suggest, the first one compares the book price against the budget of `Buyer1` and the second one compares the book price against the total budget of both `Buyer1` and `Buyer2`.

The `main` function executes the choreography using `LocalTransport` with different configurations. First, it tries to purchase `"HoTT"` with `OneBuyerDecider`. This fails because the price is higher than the budget of `Buyer1`. Then, it tries to purchase `"HoTT"` with `TwoBuyerDecider`. This succeeds because the combined budget of two buyers are bigger than the price of the book.

## `input-output.rs`

This example illustrates how to use the located input/output feature.

## `loc-poly.rs`

This example illustrates how to use the location polymorphism feature. It defines `CommAndPrint`, which is a location polymorphic choreography that moves a value from one location to another and prints it. The two locations (`sender` and `receiver`) are parametric and can be instantiated with any `ChoreographyLocation`. `MainChoreography` instantiates `CommAndPrint` with different locations.

## `runner.rs`

The `main` function of this example shows how to use the `Runner` struct to execute a choreography.

## `tic-tac-toe.rs`

This example implements a distributed tic-tac-toe game. The file contains all the necessary game logic and the choreography that orchestrates the game.

This example uses the HTTP transport to communicate between the players. To play the game, run `cargo run --example tic-tac-toe -- <PLAYER> <HOSTNAME> <PORT> <OPPONENT_HOSTNAME> <OPPONENT_PORT>`. `PLAYER` is the name of the player (`"X"` or `"O"`). `HOSTNAME` and `PORT` are the hostname and port of the player. `OPPONENT_HOSTNAME` and `OPPONENT_PORT` are the hostname and port of the opponent player. You can use the `-m` flag to use the minimax algorithm that will play the game for you. For example, you can run the following commands to play the game where player `O` will use the minimax algorithm to play the game.

```bash
# start the first player `X` on port 8080 and expect the opponent on port 8081.
cargo run --example tic-tac-toe -- X localhost 8080 localhost 8081
# start the second player `O` on port 8081 and expect the opponent on port 8080.
# `O` will use the minimax algorithm to play the game.
cargo run --example tic-tac-toe -- O localhost 8081 localhost 8080 -m
```
