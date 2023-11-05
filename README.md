
# S-Bahn Prediction

The aim of the project is to predict the future positions of S-Bahn trains. This should then help planning trips even when the trains are delayed and multiple ones need to be used to reach the destination.


## Screenshots

![App Screenshot](https://user-images.githubusercontent.com/16037346/280524162-1a675296-f789-4354-af1e-3b693e79bc9b.gif)


## Usage/Examples

### Gather Data

The following command let's you record live data into a file called `s-bahn-munich-live-map.jsonl`.

```sh
$ cargo run --bin scraper
```

### Analyze & Visualize

To analyze and visualize the following command can be used.

```sh
$ cargo run --bin analysis
```
