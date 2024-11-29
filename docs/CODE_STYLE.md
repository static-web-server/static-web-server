# Code Style

SWS uses the default configuration of `rustfmt`.

## cargo fmt

`cargo fmt --all` does not work in SWS. Please use the following commands:

### Linux/BSDs

```sh
rustfmt --check --edition 2021 $(git ls-files '*.rs')
```

### Windows

```powershell
Get-ChildItem . -Filter "*.rs" -Recurse | foreach { rustfmt --check --edition 2021 $_.FullName }
```

> **NOTE**: If you are using `rust-analyzer`, you can add the following two lines in your `settings.json` to make sure the features get taken into account when checking the project:
>
>    ```json
>     "rust-analyzer.cargo.features": ["full"],
>     "rust-analyzer.check.features": ["full"],
>    ```
