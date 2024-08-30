# Generated CLI documentation
**`SWS`** is capable of generating documentation for its command line interface in the form of man pages and shell completions.

You can generate completions for these shells and completion engines using `static-web-server generate --completions <output_path>`:
- bash
- carapace
- elvish
- fig
- fish
- nushell
- powershell
- zsh

You can generate man pages using `static-web-server generate --man-pages <output_path>`.

Finally, if you want both to be generated, you can just use `static-web-server generate <output_path>` without specifying `--completions` or `--man-pages`.
