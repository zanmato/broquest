# httpbun samples

Open this folder with Open Collection to try Broquest's `.http` support
against [httpbun.com](https://httpbun.com). Pick the `httpbun` environment.
The `local` environment expects httpbun running locally:

    docker run -p 3090:80 sharat87/httpbun

| File | Tries |
|---|---|
| `basics.http` | Methods, queries split over lines, header continuations |
| `auth.http` | Basic and Digest shorthands, Bearer, `$dotenv` |
| `bodies.http` | Urlencoded and multipart forms, `< file` and `<@ file` bodies |
| `variables.http` | File variables, variables inside variables, built in variables |
| `chaining.http` | `{{login.response...}}` references and JetBrains scripts |
| `directives.http` | `@prompt`, `@note`, `@no-redirect`, `@timeout`, `@no-log`, `>>` |
| `responses.http` | Compression, binary, slow responses |
