# Bridge stack backend

When completed, this project will be a full Rust implementation of the game
of [Bridge](https://en.wikipedia.org/wiki/Contract_bridge). It is intended to serve as a backend for either

- a web-based bridge app, or
- a smart card-replacing app to be used in a real-life setting.

### Docs

The documentation can be generated using `cargo doc`.

### Notes

1. This is an early-stage work-in-progress. It currently has a long way to go before it can be used in any sort of
   production setting. The one module that's in a reasonably finished state is the [auction](src/auction/mod.rs).

2. **TODO**: I should find a proper bridge-related name for the project, like `toofar` or `river-kwai`.