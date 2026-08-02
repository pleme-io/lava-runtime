# lava-runtime

One `EmbeddedRuntime` trait for in-process IaC DSL evaluation.

`lava-runtime` wraps [`lava-eval`](https://github.com/pleme-io/lava-eval)
(tatara-lisp), `pangea-ruby-eval` (Ruby via magnus), and — in future —
`tatara-script`, so [magma](https://github.com/pleme-io/magma) consumes any of
them through a single typed surface.

One orchestration shape for every DSL, with **zero shell-out, zero IPC, and
zero disk round-trip** between authoring and apply.

## Install

```toml
[dependencies]
lava-runtime = "0.1"
```

## The suite

```
lava-core ──┐
lava-eval ──┼──► lava-runtime
lava-schema ┤
lava-types ─┘
```

## License

MIT
