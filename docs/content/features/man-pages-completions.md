# Generated CLI documentation

**`SWS`** can generate documentation for its command-line interface through man pages and shell completions.

## Completions

You can generate completions for these shells and completion engines:

- bash
- carapace
- elvish
- fig
- fish
- nushell
- powershell
- zsh

By typing the following command, all completions will be exported to a specific directory path:


```sh
static-web-server generate --completions /my-completions-dir
```

## Man Pages

You can also generate man pages and export them to a specific directory path:

```sh
static-web-server generate --man-pages /my-man-pages-dir
```

Additionally, if you want both to be generated then just type:

```sh
static-web-server generate ./my-cli-docs-dir
```
