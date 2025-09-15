# DTS reverser

## How to use
```bash
cargo run -- <input dts> <output dts> [whitelisted tags]*
```

### Example
```bash
cargo run -- input.dts output.dts asoc-codec pinctrl
```
It will replace all ids by name of phandle owner node, where the variable name contains "asoc-codec" or "pinctrl".

### Warning!
Tag `phandle` is blacklisted.