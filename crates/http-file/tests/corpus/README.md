# Parser corpus

Every `.http` and `.rest` file here is parsed by `tests/corpus.rs`, which
checks that no byte is lost and compares the parse with the reviewed
`.parsed` summary beside it. Run `BLESS=1 cargo test -p http-file` to rewrite
the summaries after a deliberate change, then review the diff.

| Folder | Source | License |
|---|---|---|
| `broquest` | Written for this crate: line endings, continuations, scripts, VS Code variables, malformed input | Same as Broquest |
| `tree-sitter-http` | Test corpus of [rest-nvim/tree-sitter-http](https://github.com/rest-nvim/tree-sitter-http), one file per case | MIT, see `LICENSE` |
| `httprunner` | Parser fixtures of [christianhelle/httprunner](https://github.com/christianhelle/httprunner) | MIT, see `LICENSE` |
| `rest_parser` | Test data of [benfaerber/rest_parser](https://github.com/benfaerber/rest_parser) | MIT, see `LICENSE` |
| `jetbrains-spec` | Examples from the [HTTP Request in Editor Specification](https://github.com/JetBrains/http-request-in-editor-spec) | CC BY 4.0, see `NOTICE` |

httprunner and tree-sitter-http use some syntax of their own, such as
`EXPECTED_RESPONSE_STATUS` lines and pasted responses. The summaries record
how this parser reads those files, not how those tools do.
